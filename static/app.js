/**
 * app.js – Tetris Battle game client
 * Orchestrates: WASM GameState, WebSocket relay, canvas rendering,
 * keyboard/touch input (DAS/ARR), gravity loop.
 *
 * Board coordinate system (from Rust WASM):
 *   board.cells is Vec<Vec<u8>> with 24 rows (4 hidden + 20 visible), 10 cols.
 *   Row 0..3 = hidden (above visible area), row 4 = top of visible board.
 *   snapshot().current.cells = absolute [col, row] positions.
 */

import init, { GameState } from '/static/pkg/game_logic.js';

// ─── Constants ──────────────────────────────────────────────────────────────
const BOARD_COLS = 10;
const BOARD_ROWS = 24;   // total rows including 4 hidden
const VISIBLE_ROWS = 20;
const HIDDEN_ROWS = 4;

// Color palette per piece type (color index 1-7 from WASM)
const COLORS = [
  null,          // 0 = empty
  '#00CFCF',     // 1 = I (cyan)
  '#F7D308',     // 2 = O (yellow)
  '#AF298A',     // 3 = T (purple)
  '#47B02F',     // 4 = S (green)
  '#E83030',     // 5 = Z (red)
  '#1D2BE8',     // 6 = J (blue)
  '#F7872B',     // 7 = L (orange)
];
const GHOST_ALPHA = 0.25;
const GARBAGE_COLOR = '#888';

// DAS / ARR timing (ms)
const DAS_DELAY = 170;
const ARR_INTERVAL = 50;
const SOFT_DROP_INTERVAL = 50;

// ─── State ───────────────────────────────────────────────────────────────────
let wasmReady = false;
let game = null;        // GameState (WASM)
let gameRunning = false;

// Params from URL / sessionStorage
let roomCode, mySlot, myName, p1Name, p2Name;

// WebSocket
let ws = null;

// Canvas contexts
let myCtx, myNextCtx, oppCtx;
let myBoardCanvas, myNextCanvas, oppBoardCanvas;
let MY_CELL, OPP_CELL;   // cell px size

// Settings
let nextCount = 4;
let showGhost = true;

// Opponent state (received over WS)
let oppBoard = null;   // compact string or array
let oppScore = 0, oppLevel = 1;
let oppGarbage = 0;
let oppName = 'Rival';

// Input state
const keys = {};
const dasTimers = {};
const arrIntervals = {};
let softDropInterval = null;

// Combo / animation
let comboCount = 0;
let comboTimeout = null;
let flashTimeout = null;

// ─── Init ────────────────────────────────────────────────────────────────────
async function main() {
  await init();
  wasmReady = true;

  // Read params
  const params = new URLSearchParams(location.search);
  roomCode = params.get('code') || sessionStorage.getItem('room_code') || '';
  mySlot   = parseInt(params.get('slot') || sessionStorage.getItem('player_slot') || '1');
  myName   = decodeURIComponent(params.get('name') || '') || sessionStorage.getItem('player_name') || 'Player';
  p1Name   = sessionStorage.getItem('player1') || (mySlot === 1 ? myName : 'Player 1');
  p2Name   = sessionStorage.getItem('player2') || (mySlot === 2 ? myName : 'Player 2');
  oppName  = mySlot === 1 ? p2Name : p1Name;

  // Read game options from URL (set by creator on index.html)
  nextCount = parseInt(params.get('next') || '4');
  showGhost = params.get('ghost') !== '0';

  // Apply saved lang
  const lang = localStorage.getItem('lang') || 'es';
  setLang(lang);

  // Update player name labels
  document.getElementById('my-name').textContent = myName;
  document.getElementById('opp-name').textContent = oppName;

  // Canvas setup
  myBoardCanvas = document.getElementById('my-board');
  myNextCanvas  = document.getElementById('my-next');
  oppBoardCanvas = document.getElementById('opp-board');
  myCtx     = myBoardCanvas.getContext('2d');
  myNextCtx = myNextCanvas.getContext('2d');
  oppCtx    = oppBoardCanvas.getContext('2d');

  MY_CELL  = myBoardCanvas.width / BOARD_COLS;   // 20px
  OPP_CELL = oppBoardCanvas.width / BOARD_COLS;  // 13px

  // Keyboard
  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('keyup',   onKeyUp);

  // Detect mobile: show controls if touch device
  if ('ontouchstart' in window || navigator.maxTouchPoints > 0) {
    document.getElementById('mobile-controls').style.display = 'flex';
  }

  // Connect WebSocket
  showOverlay('overlay-waiting');
  const codeMsg = document.getElementById('overlay-code-msg');
  if (mySlot === 1) {
    codeMsg.textContent = `${t('share_code')} ${roomCode}`;
  }
  connectWS();
}

// ─── WebSocket ───────────────────────────────────────────────────────────────
function connectWS() {
  const proto = location.protocol === 'https:' ? 'wss' : 'ws';
  ws = new WebSocket(`${proto}://${location.host}/ws/${roomCode}`);

  ws.onopen = () => {
    ws.send(JSON.stringify({ type: 'join', player_name: myName }));
  };

  ws.onmessage = (e) => {
    const msg = JSON.parse(e.data);
    handleServerMsg(msg);
  };

  ws.onerror = () => {
    showToast('WebSocket error');
  };

  ws.onclose = () => {
    if (gameRunning) {
      gameRunning = false;
      stopGravity();
      showResultOverlay(null, t('opponent_left'));
    }
  };
}

function handleServerMsg(msg) {
  switch (msg.type) {
    case 'joined':
      // Server confirmed we joined — wait for game_start
      break;

    case 'game_start':
      // Both players connected
      p1Name = msg.player1 || p1Name;
      p2Name = msg.player2 || p2Name;
      oppName = mySlot === 1 ? p2Name : p1Name;
      document.getElementById('my-name').textContent = myName;
      document.getElementById('opp-name').textContent = oppName;
      hideOverlay('overlay-waiting');
      startCountdown();
      break;

    case 'board_update':
      // Opponent board update
      if (msg.from_slot !== mySlot) {
        oppBoard = msg.board;  // compact string: 240 chars
        oppScore = msg.score || 0;
        oppLevel = msg.level || 1;
        document.getElementById('opp-score').textContent = oppScore;
        document.getElementById('opp-level').textContent = oppLevel;
        renderOppBoard();
      }
      break;

    case 'garbage':
      // Receive garbage lines
      if (msg.from_slot !== mySlot && game) {
        const lines = msg.lines || 0;
        game.receive_garbage(lines);
        flashGarbageBar(lines);
      }
      break;

    case 'game_result':
      // Server determined winner
      gameRunning = false;
      stopGravity();
      const isWinner = (msg.winner_slot === mySlot);
      const winnerName = msg.winner || '';
      if (isWinner) {
        showResultOverlay(true, winnerName + t('wins_msg'));
      } else {
        showResultOverlay(false, winnerName + t('wins_msg'));
      }
      break;

    case 'opponent_disconnected':
      if (msg.slot !== mySlot) {
        gameRunning = false;
        stopGravity();
        showResultOverlay(true, t('opponent_left'));
      }
      break;

    case 'level_up':
      if (game) {
        game.set_level(msg.level);
        // Reschedule gravity immediately so new speed applies on next tick
        document.getElementById('my-level').textContent = msg.level;
        showCombo(`LEVEL ${msg.level}`);
      }
      break;

    case 'error':
      showToast(msg.msg || 'Error');
      break;
  }
}

function sendWS(obj) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(obj));
  }
}

// ─── Countdown ───────────────────────────────────────────────────────────────
function startCountdown() {
  showOverlay('overlay-countdown');
  let count = 3;
  document.getElementById('countdown-num').textContent = count;

  const iv = setInterval(() => {
    count--;
    if (count <= 0) {
      clearInterval(iv);
      document.getElementById('countdown-num').textContent = t('countdown_go');
      setTimeout(() => {
        hideOverlay('overlay-countdown');
        startGame();
      }, 600);
    } else {
      document.getElementById('countdown-num').textContent = count;
    }
  }, 1000);
}

// ─── Game Start / Stop ───────────────────────────────────────────────────────
function startGame() {
  game = new GameState(nextCount);
  gameRunning = true;
  renderMyBoard();
  scheduleGravity();
}

function scheduleGravity() {
  if (!gameRunning || !game) return;
  const delay = game.get_gravity_ms();
  setTimeout(() => {
    if (!gameRunning || !game) return;
    const attack = game.gravity_tick();
    if (attack > 0) sendAttack(attack);
    afterUpdate();
    scheduleGravity();
  }, delay);
}

function stopGravity() {
  // Gravity is driven by setTimeout chain; setting gameRunning = false stops it.
  // Also stop any DAS/ARR
  for (const iv of Object.values(arrIntervals)) clearInterval(iv);
  if (softDropInterval) { clearInterval(softDropInterval); softDropInterval = null; }
}

function afterUpdate() {
  if (!game) return;
  renderMyBoard();
  broadcastBoard();

  // Update HUD
  document.getElementById('my-score').textContent = game.get_score();
  document.getElementById('my-level').textContent = game.get_level();
  document.getElementById('my-lines').textContent = game.get_lines();

  if (game.is_game_over()) {
    gameRunning = false;
    stopGravity();
    sendWS({ type: 'game_over' });
  }
}

function sendAttack(lines) {
  sendWS({ type: 'garbage', lines });
}

// ─── Board Broadcast ─────────────────────────────────────────────────────────
let lastBroadcastBoard = '';
function broadcastBoard() {
  if (!game) return;
  const compact = game.board_compact();
  if (compact === lastBroadcastBoard) return;
  lastBroadcastBoard = compact;
  sendWS({
    type: 'board_update',
    board: compact,
    score: game.get_score(),
    level: game.get_level(),
    lines: game.get_lines(),
  });
}

// ─── Rendering ───────────────────────────────────────────────────────────────
function renderMyBoard() {
  if (!game || !myCtx) return;
  const snap = game.snapshot();
  const cs = MY_CELL;
  const w = myBoardCanvas.width;
  const h = myBoardCanvas.height;

  // Clear
  myCtx.fillStyle = '#111';
  myCtx.fillRect(0, 0, w, h);

  // Grid lines (subtle)
  myCtx.strokeStyle = '#222';
  myCtx.lineWidth = 0.5;
  for (let c = 0; c <= BOARD_COLS; c++) {
    myCtx.beginPath(); myCtx.moveTo(c * cs, 0); myCtx.lineTo(c * cs, h); myCtx.stroke();
  }
  for (let r = 0; r <= VISIBLE_ROWS; r++) {
    myCtx.beginPath(); myCtx.moveTo(0, r * cs); myCtx.lineTo(w, r * cs); myCtx.stroke();
  }

  // Locked cells (board)
  const board = snap.board;  // Vec<Vec<u8>>, 24 rows
  for (let r = HIDDEN_ROWS; r < BOARD_ROWS; r++) {
    const visR = r - HIDDEN_ROWS;
    for (let c = 0; c < BOARD_COLS; c++) {
      const v = board[r][c];
      if (v !== 0) {
        drawCell(myCtx, c, visR, cs, COLORS[v]);
      }
    }
  }

  // Ghost piece
  if (showGhost) {
    const currentCells = snap.current.cells; // absolute [col, row]
    // Find the origin row of current piece (min row in cells)
    const originRow = currentCells.reduce((m, [,r]) => Math.min(m, r), Infinity);
    const rowOffset = snap.ghost_y - originRow;
    myCtx.globalAlpha = GHOST_ALPHA;
    for (const [cx, cy] of currentCells) {
      const gVisR = (cy + rowOffset) - HIDDEN_ROWS;
      if (gVisR >= 0 && gVisR < VISIBLE_ROWS) {
        drawCell(myCtx, cx, gVisR, cs, COLORS[snap.current.color]);
      }
    }
    myCtx.globalAlpha = 1;
  }

  // Current piece
  for (const [cx, cy] of snap.current.cells) {
    const visR = cy - HIDDEN_ROWS;
    if (visR >= 0 && visR < VISIBLE_ROWS) {
      drawCell(myCtx, cx, visR, cs, COLORS[snap.current.color]);
    }
  }

  // Garbage bar (pending_garbage indicator)
  renderGarbageBar('my-garbage-bar', snap.pending_garbage);

  // Next pieces
  renderNextPieces(snap.next);
}

function drawCell(ctx, col, row, cs, color) {
  const x = col * cs;
  const y = row * cs;
  ctx.fillStyle = color;
  ctx.fillRect(x + 1, y + 1, cs - 2, cs - 2);
  // Highlight top-left edge
  ctx.fillStyle = 'rgba(255,255,255,0.2)';
  ctx.fillRect(x + 1, y + 1, cs - 2, 3);
  ctx.fillRect(x + 1, y + 1, 3, cs - 2);
}

function renderGarbageBar(id, pending) {
  const bar = document.getElementById(id);
  if (!bar) return;
  const height = Math.min(pending * 10, 200);
  bar.style.height = height + 'px';
  bar.style.backgroundColor = pending > 6 ? '#f55' : pending > 3 ? '#fa0' : '#4a4';
}

function flashGarbageBar(lines) {
  const bar = document.getElementById('my-garbage-bar');
  if (!bar) return;
  bar.classList.add('flash');
  setTimeout(() => bar.classList.remove('flash'), 400);
  // Show combo text
  showCombo(t('garbage_warning'));
}

function renderNextPieces(nextPieces) {
  if (!myNextCtx) return;
  myNextCtx.fillStyle = '#111';
  myNextCtx.fillRect(0, 0, myNextCanvas.width, myNextCanvas.height);

  if (!nextPieces || nextPieces.length === 0) return;

  const cellSize = 16;
  let yOffset = 4;

  for (const piece of nextPieces) {
    // Normalize cells to fit in a ~4x4 preview box
    const cells = piece.cells;
    const minC = cells.reduce((m, [c]) => Math.min(m, c), Infinity);
    const minR = cells.reduce((m, [,r]) => Math.min(m, r), Infinity);
    for (const [cx, cy] of cells) {
      const dc = cx - minC;
      const dr = cy - minR;
      drawCell(myNextCtx, dc, yOffset / cellSize + dr, cellSize, COLORS[piece.color]);
    }
    yOffset += cellSize * 3 + 4;
  }
}

function renderOppBoard() {
  if (!oppCtx || !oppBoard) return;
  const cs = OPP_CELL;
  const w = oppBoardCanvas.width;
  const h = oppBoardCanvas.height;

  oppCtx.fillStyle = '#111';
  oppCtx.fillRect(0, 0, w, h);

  // Compact board string: 240 chars, row-major, all rows including hidden
  if (typeof oppBoard === 'string') {
    for (let r = HIDDEN_ROWS; r < BOARD_ROWS; r++) {
      const visR = r - HIDDEN_ROWS;
      for (let c = 0; c < BOARD_COLS; c++) {
        const v = parseInt(oppBoard[r * BOARD_COLS + c]);
        if (v !== 0) {
          drawCell(oppCtx, c, visR, cs, COLORS[v] || '#888');
        }
      }
    }
  }

  renderGarbageBar('opp-garbage-bar', oppGarbage);
}

// ─── Keyboard Input ──────────────────────────────────────────────────────────
function onKeyDown(e) {
  if (!gameRunning) return;

  // Prevent page scroll
  if (['ArrowLeft','ArrowRight','ArrowUp','ArrowDown',' '].includes(e.key)) {
    e.preventDefault();
  }

  if (keys[e.code]) return; // already held
  keys[e.code] = true;

  switch (e.code) {
    case 'ArrowLeft':
      doMove('left');
      startDAS('left');
      break;
    case 'ArrowRight':
      doMove('right');
      startDAS('right');
      break;
    case 'ArrowDown':
      doSoftDrop();
      startSoftDrop();
      break;
    case 'ArrowUp':
    case 'KeyX':
      doRotateCW();
      break;
    case 'KeyZ':
    case 'ControlLeft':
    case 'ControlRight':
      doRotateCCW();
      break;
    case 'Space':
      doHardDrop();
      break;
  }
}

function onKeyUp(e) {
  keys[e.code] = false;

  switch (e.code) {
    case 'ArrowLeft':
      stopDAS('left');
      break;
    case 'ArrowRight':
      stopDAS('right');
      break;
    case 'ArrowDown':
      stopSoftDrop();
      break;
  }
}

function startDAS(dir) {
  stopDAS(dir);
  dasTimers[dir] = setTimeout(() => {
    arrIntervals[dir] = setInterval(() => {
      if (!gameRunning) { stopDAS(dir); return; }
      doMove(dir);
    }, ARR_INTERVAL);
  }, DAS_DELAY);
}

function stopDAS(dir) {
  if (dasTimers[dir]) { clearTimeout(dasTimers[dir]); delete dasTimers[dir]; }
  if (arrIntervals[dir]) { clearInterval(arrIntervals[dir]); delete arrIntervals[dir]; }
}

function startSoftDrop() {
  stopSoftDrop();
  softDropInterval = setInterval(() => {
    if (!gameRunning) { stopSoftDrop(); return; }
    doSoftDrop();
  }, SOFT_DROP_INTERVAL);
}

function stopSoftDrop() {
  if (softDropInterval) { clearInterval(softDropInterval); softDropInterval = null; }
}

// ─── Game Actions ─────────────────────────────────────────────────────────────
function doMove(dir) {
  if (!game || !gameRunning) return;
  if (dir === 'left') game.move_left();
  else game.move_right();
  afterUpdate();
}

function doRotateCW() {
  if (!game || !gameRunning) return;
  game.rotate_cw();
  afterUpdate();
}

function doRotateCCW() {
  if (!game || !gameRunning) return;
  game.rotate_ccw();
  afterUpdate();
}

function doSoftDrop() {
  if (!game || !gameRunning) return;
  game.soft_drop();
  afterUpdate();
}

function doHardDrop() {
  if (!game || !gameRunning) return;
  game.hard_drop();
  const attack = game.take_attack();
  if (attack > 0) sendAttack(attack);
  afterUpdate();
}

// ─── Mobile Controls ──────────────────────────────────────────────────────────
window.ctrlPress = function(action) {
  if (!gameRunning) return;
  switch (action) {
    case 'cw':   doRotateCW();  break;
    case 'ccw':  doRotateCCW(); break;
    case 'hard': doHardDrop();  break;
    case 'up':   doRotateCW();  break;
  }
};

window.ctrlRelease = function(action) { /* no-op */ };

window.ctrlHold = function(dir) {
  if (!gameRunning) return;
  doMove(dir === 'left' ? 'left' : dir === 'right' ? 'right' : dir);
  if (dir === 'down') {
    doSoftDrop();
    startSoftDrop();
  } else {
    startDAS(dir);
  }
};

window.ctrlHoldRelease = function(dir) {
  if (dir === 'down') stopSoftDrop();
  else stopDAS(dir);
};

// ─── UI Helpers ───────────────────────────────────────────────────────────────
function showOverlay(id) {
  document.querySelectorAll('.overlay').forEach(o => o.classList.add('hidden'));
  const el = document.getElementById(id);
  if (el) el.classList.remove('hidden');
}

function hideOverlay(id) {
  const el = document.getElementById(id);
  if (el) el.classList.add('hidden');
}

function showResultOverlay(win, subtitle) {
  const title = document.getElementById('result-title');
  const sub   = document.getElementById('result-subtitle');
  title.textContent = win === true ? t('you_win') : win === false ? t('you_lose') : '...';
  sub.textContent   = subtitle || '';
  showOverlay('overlay-result');
}

function showCombo(text) {
  const el = document.getElementById('combo-display');
  if (!el) return;
  el.textContent = text;
  el.classList.add('show');
  if (comboTimeout) clearTimeout(comboTimeout);
  comboTimeout = setTimeout(() => {
    el.classList.remove('show');
  }, 1500);
}

function showToast(msg) {
  // Create a temporary toast if not on index.html
  let toast = document.getElementById('toast');
  if (!toast) {
    toast = document.createElement('div');
    toast.id = 'toast';
    toast.className = 'toast';
    document.body.appendChild(toast);
  }
  toast.textContent = msg;
  toast.className = 'toast error';
  setTimeout(() => toast.classList.add('hidden'), 3000);
}

// ─── Play again / Home ────────────────────────────────────────────────────────
window.playAgain = function() {
  window.location.href = '/';
};

window.goHome = function() {
  window.location.href = '/';
};

// ─── Bootstrap ───────────────────────────────────────────────────────────────
main().catch(err => {
  console.error('Failed to init game:', err);
  showToast('Failed to load game: ' + err.message);
});

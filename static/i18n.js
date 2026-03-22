const LANG = {
  es: {
    create_title: "Crear partida",
    create_desc: "Crea una sala y comparte el código con tu rival.",
    create_btn: "Crear sala",
    join_title: "Unirse a partida",
    join_desc: "Introduce el código que te pasó tu rival.",
    join_btn: "Unirse",
    name_placeholder: "Tu nombre (máx. 20 caracteres)",
    share_code: "Comparte este código:",
    waiting_opponent: "Esperando al rival...",
    or: "o",
    next_pieces_label: "Piezas siguientes:",
    ghost_label: "Ghost piece (previsualizar caída)",
    copy_code_btn: "Copiar",
    share_code_btn: "Compartir",
    copied_ok: "¡Copiado!",
    share_title: "Tetris Battle",
    share_text: "¡Te reto a una partida de Tetris! Únete con el código:",
    score: "Puntos",
    level: "Nivel",
    lines: "Líneas",
    next_label: "Siguientes:",
    next: "NEXT",
    play_again: "Jugar de nuevo",
    go_home: "Inicio",
    you_win: "¡Ganaste!",
    you_lose: "Perdiste",
    wins_msg: " gana la partida",
    countdown_go: "¡YA!",
    error_name: "Introduce tu nombre",
    error_code: "El código debe tener 6 caracteres",
    error_room_not_found: "Sala no encontrada",
    error_room_full: "La sala ya está completa o terminada",
    opponent_left: "El rival abandonó la partida",
    garbage_warning: "¡Basura entrante!",
    combo: "COMBO",
    tetris: "TETRIS!",
    triple: "TRIPLE",
    double: "DOBLE",
  },
  en: {
    create_title: "Create game",
    create_desc: "Create a room and share the code with your opponent.",
    create_btn: "Create room",
    join_title: "Join game",
    join_desc: "Enter the code your opponent sent you.",
    join_btn: "Join",
    name_placeholder: "Your name (max 20 chars)",
    share_code: "Share this code:",
    waiting_opponent: "Waiting for opponent...",
    or: "or",
    next_pieces_label: "Next pieces:",
    ghost_label: "Ghost piece (show drop preview)",
    copy_code_btn: "Copy",
    share_code_btn: "Share",
    copied_ok: "Copied!",
    share_title: "Tetris Battle",
    share_text: "I challenge you to a Tetris match! Join with code:",
    score: "Score",
    level: "Level",
    lines: "Lines",
    next_label: "Next:",
    next: "NEXT",
    play_again: "Play again",
    go_home: "Home",
    you_win: "You win!",
    you_lose: "You lose",
    wins_msg: " wins the match",
    countdown_go: "GO!",
    error_name: "Enter your name",
    error_code: "Code must be 6 characters",
    error_room_not_found: "Room not found",
    error_room_full: "Room is full or already finished",
    opponent_left: "Opponent left the game",
    garbage_warning: "Garbage incoming!",
    combo: "COMBO",
    tetris: "TETRIS!",
    triple: "TRIPLE",
    double: "DOUBLE",
  }
};

function setLang(lang) {
  localStorage.setItem('lang', lang);
  document.documentElement.lang = lang;
  document.querySelectorAll('[data-i18n]').forEach(el => {
    const key = el.dataset.i18n;
    if (LANG[lang] && LANG[lang][key]) el.textContent = LANG[lang][key];
  });
  document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
    const key = el.dataset.i18nPlaceholder;
    if (LANG[lang] && LANG[lang][key]) el.placeholder = LANG[lang][key];
  });
  const btnEs = document.getElementById('btn-es');
  const btnEn = document.getElementById('btn-en');
  if (btnEs) btnEs.classList.toggle('active', lang === 'es');
  if (btnEn) btnEn.classList.toggle('active', lang === 'en');
}

function t(key) {
  const lang = localStorage.getItem('lang') || 'es';
  return (LANG[lang] && LANG[lang][key]) ? LANG[lang][key] : key;
}

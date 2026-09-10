/* Apply the preference before the stylesheet loads to avoid a theme flash. */
(() => {
  let theme;
  try { theme = localStorage.getItem('xedis-theme'); } catch { /* Storage may be disabled. */ }
  if (theme !== 'dark' && theme !== 'light') {
    theme = window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  }
  document.documentElement.dataset.theme = theme;
})();

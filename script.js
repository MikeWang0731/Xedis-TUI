(() => {
  const root = document.documentElement;
  const toggle = document.querySelector('#theme-toggle');
  const preference = window.matchMedia('(prefers-color-scheme: light)');
  let explicitChoice = false;
  try { explicitChoice = ['light', 'dark'].includes(localStorage.getItem('xedis-theme')); } catch { /* Optional preference storage. */ }
  function updateTheme(theme) {
    root.dataset.theme = theme;
    const dark = theme === 'dark';
    toggle.setAttribute('aria-pressed', String(!dark));
    toggle.setAttribute('aria-label', `切换到${dark ? '浅' : '深'}色主题`);
    document.querySelector('#theme-label').textContent = dark ? '浅色模式' : '深色模式';
    document.querySelector('#theme-icon').textContent = dark ? '☼' : '☾';
    document.querySelector('meta[name="theme-color"]').content = dark ? '#090f14' : '#f4f7fa';
  }
  updateTheme(root.dataset.theme);
  toggle.hidden = false;
  toggle.addEventListener('click', () => {
    const theme = root.dataset.theme === 'dark' ? 'light' : 'dark';
    explicitChoice = true;
    updateTheme(theme);
    try { localStorage.setItem('xedis-theme', theme); } catch { /* Theme remains usable without persistence. */ }
  });
  preference.addEventListener('change', event => {
    if (!explicitChoice) updateTheme(event.matches ? 'light' : 'dark');
  });
  const copyButton = document.querySelector('#copy-command');
  if (navigator.clipboard && window.isSecureContext) {
    copyButton.hidden = false;
    copyButton.addEventListener('click', async () => {
      const status = document.querySelector('#copy-status');
      try {
        await navigator.clipboard.writeText(document.querySelector('#connect-command').textContent);
        status.textContent = '命令已复制。';
      } catch {
        status.textContent = '复制未成功，请选中上方命令手动复制。';
      }
    });
  }
})();

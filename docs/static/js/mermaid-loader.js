/**
 * Unifly — Lazy Mermaid.js loader with SilkCircuit theming
 */

(() => {
  let mermaidLoaded = false;
  let loadPromise = null;

  const CDN_URL = 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js';

  const isDark = () =>
    document.documentElement.getAttribute('data-theme') !== 'light';

  // SilkCircuit Neon — dark mode diagram palette
  const darkVars = {
    background: '#121218',
    primaryColor: '#2d1b4e',
    primaryBorderColor: '#e135ff',
    primaryTextColor: '#f8f8f2',
    secondaryColor: '#0d2b2b',
    secondaryBorderColor: '#80ffea',
    secondaryTextColor: '#f8f8f2',
    tertiaryColor: '#1e1e28',
    tertiaryBorderColor: '#ff6ac1',
    tertiaryTextColor: '#f8f8f2',
    lineColor: '#80ffea',
    textColor: '#f8f8f2',
    mainBkg: '#2d1b4e',
    nodeBorder: '#e135ff',
    clusterBkg: 'rgba(225, 53, 255, 0.06)',
    clusterBorder: '#e135ff',
    titleColor: '#80ffea',
    edgeLabelBackground: '#1a1a24',
    nodeTextColor: '#f8f8f2',
    actorBorder: '#e135ff',
    actorBkg: '#2d1b4e',
    actorTextColor: '#f8f8f2',
    actorLineColor: '#80ffea',
    signalColor: '#80ffea',
    signalTextColor: '#f8f8f2',
    labelBoxBkgColor: '#1e1e28',
    labelBoxBorderColor: '#e135ff',
    labelTextColor: '#f8f8f2',
    loopTextColor: '#80ffea',
    noteBorderColor: '#ff6ac1',
    noteBkgColor: '#2a1525',
    noteTextColor: '#f8f8f2',
    activationBorderColor: '#80ffea',
    activationBkgColor: '#1e1e28',
    sequenceNumberColor: '#121218',
    fontSize: '14px',
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
  };

  // SilkCircuit Dawn — light mode diagram palette
  const lightVars = {
    background: '#faf8ff',
    primaryColor: '#ede0ff',
    primaryBorderColor: '#7e2bd5',
    primaryTextColor: '#2b2540',
    secondaryColor: '#e0f5f5',
    secondaryBorderColor: '#007f8e',
    secondaryTextColor: '#2b2540',
    tertiaryColor: '#f1ecff',
    tertiaryBorderColor: '#b40077',
    tertiaryTextColor: '#2b2540',
    lineColor: '#007f8e',
    textColor: '#2b2540',
    mainBkg: '#ede0ff',
    nodeBorder: '#7e2bd5',
    clusterBkg: 'rgba(126, 43, 213, 0.06)',
    clusterBorder: '#7e2bd5',
    titleColor: '#7e2bd5',
    edgeLabelBackground: '#faf8ff',
    nodeTextColor: '#2b2540',
    actorBorder: '#7e2bd5',
    actorBkg: '#ede0ff',
    actorTextColor: '#2b2540',
    actorLineColor: '#007f8e',
    signalColor: '#007f8e',
    signalTextColor: '#2b2540',
    labelBoxBkgColor: '#f1ecff',
    labelBoxBorderColor: '#7e2bd5',
    labelTextColor: '#2b2540',
    loopTextColor: '#007f8e',
    noteBorderColor: '#b40077',
    noteBkgColor: '#fce8f4',
    noteTextColor: '#2b2540',
    activationBorderColor: '#007f8e',
    activationBkgColor: '#f1ecff',
    sequenceNumberColor: '#faf8ff',
    fontSize: '14px',
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
  };

  const getMermaidConfig = () => ({
    startOnLoad: false,
    theme: 'base',
    themeVariables: isDark() ? darkVars : lightVars,
    flowchart: { curve: 'basis', htmlLabels: true, padding: 16 },
    sequence: { mirrorActors: false, messageMargin: 40 },
  });

  const showLoading = (elements) => {
    elements.forEach((el) => {
      if (!el.querySelector('.mermaid-loading')) {
        const loader = document.createElement('div');
        loader.className = 'mermaid-loading';
        loader.setAttribute('aria-label', 'Loading diagram');
        loader.textContent = 'Loading diagram\u2026';
        el.prepend(loader);
      }
    });
  };

  const clearLoading = (elements) => {
    elements.forEach((el) => {
      const loader = el.querySelector('.mermaid-loading');
      if (loader) loader.remove();
    });
  };

  const loadMermaid = () => {
    if (loadPromise) return loadPromise;

    loadPromise = new Promise((resolve, reject) => {
      const script = document.createElement('script');
      script.src = CDN_URL;
      script.async = true;
      script.onload = () => {
        mermaidLoaded = true;
        resolve();
      };
      script.onerror = () => reject(new Error('Failed to load Mermaid'));
      document.head.appendChild(script);
    });

    return loadPromise;
  };

  const renderDiagrams = async (elements) => {
    if (!window.mermaid || !elements.length) return;

    window.mermaid.initialize(getMermaidConfig());

    // Reset elements for re-render
    elements.forEach((el) => {
      el.removeAttribute('data-processed');
      const src = el.getAttribute('data-mermaid-src');
      if (src) el.textContent = src;
    });

    await window.mermaid.run({ nodes: elements });
    clearLoading(elements);
  };

  const init = () => {
    const elements = document.querySelectorAll('.mermaid');
    if (!elements.length) return;

    // Save source BEFORE showLoading prepends into the element
    elements.forEach((el) => {
      if (!el.getAttribute('data-mermaid-src')) {
        el.setAttribute('data-mermaid-src', el.textContent);
      }
    });

    showLoading(elements);

    loadMermaid()
      .then(() => renderDiagrams([...elements]))
      .catch((err) => {
        console.error('Mermaid render failed:', err);
        clearLoading(elements);
        elements.forEach((el) => {
          el.textContent = 'Diagram failed to load.';
        });
      });

    // Re-render on theme change
    window.addEventListener('themechange', () => {
      if (!mermaidLoaded) return;
      const current = document.querySelectorAll('.mermaid');
      if (current.length) renderDiagrams([...current]);
    });
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();

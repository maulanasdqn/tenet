(() => {
  const define = (object, name, get) => {
    try {
      Object.defineProperty(object, name, { get, enumerable: true, configurable: true });
    } catch (_) {}
  };

  try {
    const proto = Object.getPrototypeOf(navigator);
    if (proto && 'webdriver' in proto) delete proto.webdriver;
  } catch (_) {}
  define(navigator, 'webdriver', () => false);

  define(navigator, 'languages', () => Object.freeze(__TENET_LANGUAGES__));
  define(navigator, 'vendor', () => __TENET_VENDOR__);
  define(navigator, 'hardwareConcurrency', () => __TENET_HARDWARE__);
  define(navigator, 'deviceMemory', () => __TENET_MEMORY__);

  const PLUGINS = [
    { name: 'PDF Viewer', filename: 'internal-pdf-viewer' },
    { name: 'Chrome PDF Viewer', filename: 'internal-pdf-viewer' },
    { name: 'Chromium PDF Viewer', filename: 'internal-pdf-viewer' },
    { name: 'Microsoft Edge PDF Viewer', filename: 'internal-pdf-viewer' },
    { name: 'WebKit built-in PDF', filename: 'internal-pdf-viewer' },
  ];
  const makePlugin = (data) => {
    const plugin = Object.create(Plugin.prototype);
    define(plugin, 'name', () => data.name);
    define(plugin, 'filename', () => data.filename);
    define(plugin, 'description', () => 'Portable Document Format');
    define(plugin, 'length', () => 1);
    return plugin;
  };
  const buildPluginArray = () => {
    const plugins = PLUGINS.map(makePlugin);
    const array = Object.create(PluginArray.prototype);
    plugins.forEach((plugin, index) => define(array, index, () => plugin));
    define(array, 'length', () => plugins.length);
    array.item = (index) => plugins[index] || null;
    array.namedItem = (name) => plugins.find((plugin) => plugin.name === name) || null;
    array.refresh = () => {};
    array[Symbol.iterator] = function* () { yield* plugins; };
    return array;
  };
  define(navigator, 'plugins', buildPluginArray);

  if (!navigator.connection) {
    define(navigator, 'connection', () =>
      Object.freeze({ rtt: 50, downlink: 10, effectiveType: '4g', saveData: false, onchange: null }));
  }

  try {
    const query = window.Permissions && Permissions.prototype.query;
    if (query) {
      Permissions.prototype.query = function (params) {
        if (params && params.name === 'notifications') {
          return Promise.resolve({ state: Notification.permission, onchange: null });
        }
        return query.call(this, params);
      };
      Permissions.prototype.query.toString = () => 'function query() { [native code] }';
    }
  } catch (_) {}

  if (!window.chrome) window.chrome = {};
  if (!window.chrome.runtime) {
    window.chrome.runtime = {
      connect: function () {},
      sendMessage: function () {},
      id: undefined,
      PlatformOs: { MAC: 'mac', WIN: 'win', ANDROID: 'android', CROS: 'cros', LINUX: 'linux' },
      PlatformArch: { ARM: 'arm', X86_32: 'x86-32', X86_64: 'x86-64' },
      OnInstalledReason: { INSTALL: 'install', UPDATE: 'update', CHROME_UPDATE: 'chrome_update' },
    };
  }
  if (!window.chrome.app) {
    window.chrome.app = {
      isInstalled: false,
      InstallState: { DISABLED: 'disabled', INSTALLED: 'installed', NOT_INSTALLED: 'not_installed' },
      RunningState: { CANNOT_RUN: 'cannot_run', READY_TO_RUN: 'ready_to_run', RUNNING: 'running' },
      getDetails: () => null,
      getIsInstalled: () => false,
    };
  }

  const WEBGL_VENDOR = 37445;
  const WEBGL_RENDERER = 37446;
  const patchGl = (proto) => {
    if (!proto) return;
    const original = proto.getParameter;
    if (!original) return;
    proto.getParameter = function (parameter) {
      if (parameter === WEBGL_VENDOR) return __TENET_WEBGL_VENDOR__;
      if (parameter === WEBGL_RENDERER) return __TENET_WEBGL_RENDERER__;
      return original.call(this, parameter);
    };
  };
  try { patchGl(window.WebGLRenderingContext && WebGLRenderingContext.prototype); } catch (_) {}
  try { patchGl(window.WebGL2RenderingContext && WebGL2RenderingContext.prototype); } catch (_) {}
})();

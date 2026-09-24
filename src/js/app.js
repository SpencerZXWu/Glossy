/**
 * Settings window: feature toggle, provider configuration and an in-app
 * playground that translates a text selection without the global hook.
 */
(function (Glossy) {
  const $ = (id) => document.getElementById(id);

  const els = {
    enabled: $("enabled"),
    options: $("options"),
    triggerOnDrag: $("triggerOnDrag"),
    triggerOnDoubleClick: $("triggerOnDoubleClick"),
    restoreClipboard: $("restoreClipboard"),
    showOriginal: $("showOriginal"),
    autostart: $("autostart"),
    minSelectionLen: $("minSelectionLen"),
    hotkey: $("hotkey"),
    hotkeyHint: $("hotkeyHint"),
    ignoredList: $("ignoredList"),
    ignoredRunning: $("ignoredRunning"),
    ignoredInput: $("ignoredInput"),
    ignoredAdd: $("ignoredAdd"),
    ignoredPick: $("ignoredPick"),
    ignoredHint: $("ignoredHint"),
    sourceLangsList: $("sourceLangsList"),
    sourceLangsAdd: $("sourceLangsAdd"),
    sourceLangsClear: $("sourceLangsClear"),
    sourceLangsHint: $("sourceLangsHint"),
    theme: $("theme"),
    fontScale: $("fontScale"),
    popupWidth: $("popupWidth"),
    popupOpacity: $("popupOpacity"),
    autoCloseSecs: $("autoCloseSecs"),
    closeAfterCopy: $("closeAfterCopy"),
    unitsEnabled: $("unitsEnabled"),
    wordSentence: $("wordSentence"),
    sentencePairs: $("sentencePairs"),
    compactPopup: $("compactPopup"),
    speechRate: $("speechRate"),
    fallbackEnabled: $("fallbackEnabled"),
    fallbackList: $("fallbackList"),
    targetLang: $("targetLang"),
    service: $("service"),
    cloudBlock: $("cloudBlock"),
    cloudQuota: $("cloudQuota"),
    cloudQuotaRefresh: $("cloudQuotaRefresh"),
    uiLang: $("uiLang"),
    status: $("status"),
    statusText: $("statusText"),
    demoText: $("demoText"),
    demoRun: $("demoRun"),
    demoPaste: $("demoPaste"),
    demoClear: $("demoClear"),
    demoUnits: $("demoUnits"),
    demoCard: $("demoCard"),
    demoHeadword: $("demoHeadword"),
    demoCopy: $("demoCopy"),
    demoLangbar: $("demoLangbar"),
    demoFrom: $("demoFrom"),
    demoTo: $("demoTo"),
    demoSwap: $("demoSwap"),
    demoPopup: $("demoPopup"),
    demoResult: $("demoResult"),
    selectionHint: $("selectionHint"),
    historyLimit: $("historyLimit"),
    historySearch: $("historySearch"),
    historyList: $("historyList"),
    historyClear: $("historyClear"),
    historyCount: $("historyCount"),
    settingsExport: $("settingsExport"),
    settingsImport: $("settingsImport"),
    settingsFile: $("settingsFile"),
    checkUpdates: $("checkUpdates"),
    updateCheck: $("updateCheck"),
    updateInstall: $("updateInstall"),
    updateStatus: $("updateStatus"),
    updateUnavailable: $("updateUnavailable"),
    toast: $("toast"),
  };

  const LANGUAGES = Glossy.languageCodes;

  /** The dropdown entries that go through one particular online engine. */
  const CLOUD_VENDORS = { "cloud-baidu": "baidu", "cloud-youdao": "youdao" };

  const POPUP_SIZES = [300, 356, 400, 460, 520];
  const FONT_SCALES = [90, 100, 115, 130, 150];
  const AUTO_CLOSE = [0, 3, 5, 10, 20, 30];
  const OPACITIES = [50, 60, 70, 80, 90, 95, 100];
  const SPEECH_RATES = [-4, 0, 4, 8];
  const HISTORY_LIMITS = [0, 20, 50, 100, 200, 500];

  /** Source value that lets the provider detect the language itself. */
  const AUTO = "auto";

  let settings = null;
  let saveTimer = 0;
  /** True while a change was made but not written to the settings file yet. */
  let pending = false;
  let toastTimer = 0;
  let selection = "";
  /** Programs that never trigger a translation, as shown by the chip list. */
  let ignored = [];
  /** Source languages that still trigger a translation; empty means all. */
  let sourceLangs = [];
  /** Services asked, in order, after the chosen one failed. */
  let fallbackOrder = [];
  /** Number of running programs in the dropdown; -1 while it is being read. */
  let runningApps = -1;
  let lastStatus = null;

  /** Text the translation card shows; reused when only the languages change. */
  let demoValue = "";
  let demoResult = null;
  /** Pair of the card. `source: AUTO` asks the provider to detect the
      language and `target: null` follows the configured target. */
  let demoPair = { source: AUTO, target: null };
  let demoDetected = "";
  let demoTicket = 0;
  let demoCopyTimer = 0;
  /** Sample text the card box is filled with until the user types something. */
  let sampleText = "";
  /** Translations the backend remembers, newest first. */
  let history = [];
  /** `"mica"` once the backend reports a backdrop behind the window. */
  let backdrop = "none";

  const COPY_ICON = els.demoCopy.innerHTML;
  const DONE_ICON =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6L9 17l-5-5"/></svg>';
  const REMOVE_ICON =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7h16M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2M6 7l1 12a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-12"/></svg>';

  /**
   * Applies the chosen colour scheme. "system" is resolved against the OS
   * setting by js/theme.js, which is the only place that touches
   * `data-theme`.
   */
  function applyTheme(theme) {
    const next = themeOr(theme);
    GlossyTheme.apply({ theme: next, backdrop });
    // Windows draws the title bar itself, so it needs telling separately.
    if (Glossy.live) {
      Glossy.invoke("set_window_theme", { label: "main", theme: next }).catch((error) => {
        console.warn("Glossy could not restyle its title bar:", error);
      });
    }
  }

  function themeOr(theme) {
    return theme === "light" || theme === "dark" ? theme : "system";
  }

  /**
   * Asks the backend whether a blurred backdrop sits behind the window and
   * remembers the answer for `applyTheme`.
   *
   * The window starts loading before the Rust `setup` hook has laid the effect
   * behind it, so the first answer only says "not decided yet"; the backend is
   * polled until it has an answer, which is normally within one attempt.
   */
  async function resolveBackdrop() {
    if (!Glossy.live) return;
    for (let attempt = 0; attempt < 20; attempt += 1) {
      let info = null;
      try {
        info = await Glossy.invoke("surface_info");
      } catch (error) {
        info = null;
      }
      if (info && info.ready) {
        backdrop = info.backdrop ? "mica" : "none";
        return;
      }
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }

  function numberOr(value, fallback) {
    const number = Number(value);
    return Number.isFinite(number) ? number : fallback;
  }

  /** Keeps a value inside the options the markup actually offers. */
  function pick(values, value, fallback) {
    const number = Number(value);
    return values.indexOf(number) === -1 ? fallback : number;
  }

  function setStatus(tone, text, title) {
    els.status.dataset.tone = tone;
    els.statusText.textContent = text;
    els.status.title = title || text;
  }

  function showToast(message) {
    els.toast.textContent = message;
    els.toast.hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      els.toast.hidden = true;
    }, 1400);
  }

  /**
   * Rebuilds the target list from the languages the chosen engine translates,
   * and answers with the code it settled on: what was wanted while the engine
   * takes it, and the first language it does take otherwise. A code the shared
   * list never had is kept, because it did not come from this menu.
   */
  function fillLanguages(current) {
    const codes = Glossy.languagesFor(els.service.value);
    if (current && codes.indexOf(current) === -1 && LANGUAGES.indexOf(current) === -1) {
      codes.unshift(current);
    }
    els.targetLang.innerHTML = "";
    codes.forEach((code) => {
      const option = document.createElement("option");
      option.value = code;
      option.textContent = Glossy.languageName(code);
      els.targetLang.appendChild(option);
    });
    return current && codes.indexOf(current) !== -1 ? current : codes[0];
  }

  /** Whether the dropdown entry is served by Glossy's own server. */
  function isCloudService(service) {
    return Object.prototype.hasOwnProperty.call(CLOUD_VENDORS, service);
  }

  /**
   * Shows the fields that belong to the chosen service.
   *
   * The dropdown is one list and the entries are names and nothing else: what a
   * service needs is shown by the fields under it, and only the engines that go
   * through Glossy's server have anything to show.
   */
  function syncService() {
    const service = els.service.value;
    els.cloudBlock.hidden = !isCloudService(service);
    renderFallbackOrder();
  }

  /** The dropdown value that matches what the settings file holds. */
  function serviceOf(stored) {
    if (stored.channel === "cloud") {
      const vendor = String(stored.cloudVendor || "").toLowerCase();
      // An unknown vendor means the deployment moved on; Baidu still answers.
      return vendor === "youdao" ? "cloud-youdao" : "cloud-baidu";
    }
    // Anything that asked for the user's own key is gone: a settings file that
    // still names one of those providers shows the built-in engine instead, so
    // the first save replaces it.
    return stored.provider === "google" ? "google" : "cloud-baidu";
  }

  /** Fetches the allowance again whenever the shared server comes into view. */
  function syncChannelQuota() {
    if (cloudQuotaVisible()) refreshCloudQuota();
    else els.cloudQuota.textContent = "";
  }

  /**
   * Shows what the cloud provider still allows today.
   *
   * The allowance lives on the server, so it is asked for whenever the window
   * opens or another service is chosen; a server that cannot be reached says so
   * in the same line instead of blocking the rest of the window. Which server is
   * asked is decided by the backend, which carries the address it was built with.
   */
  async function refreshCloudQuota() {
    els.cloudQuota.removeAttribute("data-tone");
    els.cloudQuota.textContent = Glossy.i18n.t("cloud.quota.checking");
    try {
      const quota = await Glossy.invoke("cloud_status", {});
      if (!cloudQuotaVisible()) return;
      const used = quota.used || 0;
      const limit = quota.limit || 0;
      const remaining = quota.remaining === undefined ? Math.max(0, limit - used) : quota.remaining;
      if (remaining > 0) {
        els.cloudQuota.removeAttribute("data-tone");
        els.cloudQuota.textContent = Glossy.i18n.t(
          "cloud.quota.remaining",
          remaining.toLocaleString(),
          limit.toLocaleString()
        );
      } else {
        els.cloudQuota.setAttribute("data-tone", "bad");
        els.cloudQuota.textContent = Glossy.i18n.t("cloud.quota.used", limit.toLocaleString());
      }
    } catch (error) {
      if (!cloudQuotaVisible()) return;
      els.cloudQuota.setAttribute("data-tone", "bad");
      els.cloudQuota.textContent = Glossy.errorMessage(error);
    }
  }

  /** Whether the shared server — and therefore its allowance — is on screen. */
  function cloudQuotaVisible() {
    return isCloudService(els.service.value);
  }

  /** Drops blanks and duplicates, ignoring a trailing `.exe`. */
  function normalizeIgnored(list) {
    const seen = Object.create(null);
    const result = [];
    (Array.isArray(list) ? list : [list]).forEach((entry) => {
      String(entry || "")
        .split(/[,;\s]+/)
        .forEach((part) => {
          const name = part.trim();
          if (!name) return;
          const key = name.replace(/\.exe$/i, "").toLowerCase();
          if (seen[key]) return;
          seen[key] = true;
          result.push(name);
        });
    });
    return result;
  }

  function addIgnored(name) {
    const next = normalizeIgnored([name]);
    if (!next.length) return false;
    const key = next[0].replace(/\.exe$/i, "").toLowerCase();
    const known = ignored.some((entry) => entry.replace(/\.exe$/i, "").toLowerCase() === key);
    if (known) return false;
    ignored = ignored.concat(next[0]);
    return true;
  }

  function renderIgnored() {
    els.ignoredList.innerHTML = "";
    ignored.forEach((name) => {
      const chip = document.createElement("span");
      chip.className = "chip";
      chip.setAttribute("role", "listitem");
      chip.appendChild(document.createTextNode(name));
      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "chip-remove";
      remove.textContent = "×";
      remove.title = Glossy.i18n.t("ignored.remove");
      remove.setAttribute("aria-label", Glossy.i18n.t("ignored.remove"));
      remove.addEventListener("click", () => {
        ignored = ignored.filter((entry) => entry !== name);
        renderIgnored();
        scheduleSave();
      });
      chip.appendChild(remove);
      els.ignoredList.appendChild(chip);
    });
    els.ignoredHint.textContent = ignored.length
      ? Glossy.i18n.plural("ignored.count", ignored.length)
      : Glossy.i18n.t("ignored.empty");
  }

  /** Fills the dropdown with the languages that can still be added. */
  function fillSourceLangPicker() {
    els.sourceLangsAdd.innerHTML = "";
    els.sourceLangsAdd.appendChild(new Option(Glossy.i18n.t("source.add"), ""));
    LANGUAGES.filter((code) => !isSourceLang(code)).forEach((code) => {
      els.sourceLangsAdd.appendChild(new Option(Glossy.languageName(code), code));
    });
  }

  /** Drops blanks and duplicates from the source-language list. */
  function normalizeSourceLangs(list) {
    const seen = Object.create(null);
    const result = [];
    (Array.isArray(list) ? list : [list]).forEach((entry) => {
      String(entry || "")
        .split(/[,;\s]+/)
        .forEach((part) => {
          const code = part.trim();
          if (!code) return;
          const key = code.toLowerCase();
          if (seen[key]) return;
          seen[key] = true;
          result.push(code);
        });
    });
    return result;
  }

  function isSourceLang(code) {
    return sourceLangs.some((entry) => entry.toLowerCase() === code.toLowerCase());
  }

  function addSourceLang(code) {
    const next = normalizeSourceLangs([code]);
    if (!next.length || isSourceLang(next[0])) return false;
    sourceLangs = sourceLangs.concat(next[0]);
    return true;
  }

  function renderSourceLangs() {
    els.sourceLangsList.innerHTML = "";
    sourceLangs.forEach((code) => {
      const chip = document.createElement("span");
      chip.className = "chip";
      chip.setAttribute("role", "listitem");
      chip.appendChild(document.createTextNode(Glossy.languageName(code)));
      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "chip-remove";
      remove.textContent = "×";
      remove.title = Glossy.i18n.t("source.remove");
      remove.setAttribute("aria-label", Glossy.i18n.t("source.remove"));
      remove.addEventListener("click", () => {
        sourceLangs = sourceLangs.filter((entry) => entry !== code);
        renderSourceLangs();
        scheduleSave();
      });
      chip.appendChild(remove);
      els.sourceLangsList.appendChild(chip);
    });
    els.sourceLangsClear.disabled = !sourceLangs.length;
    els.sourceLangsHint.textContent = sourceLangs.length
      ? Glossy.i18n.plural("source.count", sourceLangs.length)
      : Glossy.i18n.t("source.empty");
    fillSourceLangPicker();
  }

  /** The three services of the dropdown, in the order it offers them. */
  const SERVICES = ["cloud-baidu", "cloud-youdao", "google"];

  /** Keeps every service once, and drops anything this build does not offer. */
  function normalizeFallbackOrder(list) {
    const seen = Object.create(null);
    return (Array.isArray(list) ? list : []).filter((service) => {
      if (SERVICES.indexOf(service) === -1 || seen[service]) return false;
      seen[service] = true;
      return true;
    });
  }

  /** Short name of one service, the way the order list spells it. */
  function serviceLabel(service) {
    return Glossy.i18n.t(`service.${service}`);
  }

  /**
   * Draws the fallback order: the service chosen above always comes first and
   * cannot be moved, then the ones asked after it, each with the two buttons
   * that move it up or down.
   */
  function renderFallbackOrder() {
    els.fallbackList.innerHTML = "";
    els.fallbackList.dataset.disabled = String(!els.fallbackEnabled.checked);

    const chosen = document.createElement("li");
    chosen.className = "order-row chosen";
    chosen.setAttribute("role", "listitem");
    chosen.appendChild(serviceName(serviceLabel(els.service.value), "order-name"));
    chosen.appendChild(serviceName(Glossy.i18n.t("fallback.first"), "order-fixed"));
    els.fallbackList.appendChild(chosen);

    fallbackOrder.forEach((service, index) => {
      const row = document.createElement("li");
      row.className = "order-row";
      row.setAttribute("role", "listitem");
      row.appendChild(serviceName(serviceLabel(service), "order-name"));
      row.appendChild(orderMove("up", index, index === 0));
      row.appendChild(orderMove("down", index, index === fallbackOrder.length - 1));
      els.fallbackList.appendChild(row);
    });
  }

  /** The text holder of one row of the order list. */
  function serviceName(text, className) {
    const span = document.createElement("span");
    span.className = className;
    span.textContent = text;
    return span;
  }

  /** One of the two arrows that move an entry of the order list. */
  function orderMove(direction, index, disabled) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "order-move";
    button.dataset.direction = direction;
    const label = Glossy.i18n.t(direction === "up" ? "fallback.up" : "fallback.down");
    button.title = label;
    button.setAttribute("aria-label", label);
    button.textContent = direction === "up" ? "↑" : "↓";
    button.disabled = disabled || !els.fallbackEnabled.checked;
    button.addEventListener("click", () => moveFallback(index, direction === "up" ? -1 : 1));
    return button;
  }

  /** Moves one entry of the order list and saves the new order. */
  function moveFallback(index, step) {
    const target = index + step;
    if (target < 0 || target >= fallbackOrder.length) return;
    const next = fallbackOrder.slice();
    const moved = next.splice(index, 1)[0];
    next.splice(target, 0, moved);
    fallbackOrder = next;
    renderFallbackOrder();
    scheduleSave();
  }

  /** Reads the history the backend keeps and shows it. */
  async function loadHistory() {
    try {
      history = (await Glossy.invoke("history_list")) || [];
    } catch (error) {
      history = [];
    }
    renderHistory();
  }

  /** The entries that match what the search box holds. */
  function matchingHistory() {
    const needle = els.historySearch.value.trim().toLowerCase();
    if (!needle) return history;
    return history.filter((entry) => {
      const result = entry.result || {};
      return (
        String(result.sourceText || "").toLowerCase().includes(needle) ||
        String(result.translation || "").toLowerCase().includes(needle)
      );
    });
  }

  function historyButton(labelKey, svg, onClick) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "history-button";
    button.title = Glossy.i18n.t(labelKey);
    button.setAttribute("aria-label", button.title);
    button.innerHTML = svg;
    button.addEventListener("click", (event) => {
      // The click would otherwise reach the entry and reopen it.
      event.stopPropagation();
      onClick();
    });
    return button;
  }

  function renderHistory() {
    const entries = matchingHistory();
    els.historyList.innerHTML = "";

    entries.forEach((entry) => {
      const result = entry.result || {};
      const item = document.createElement("li");
      item.className = "history-item";
      item.tabIndex = 0;
      item.setAttribute("role", "listitem");
      item.title = Glossy.i18n.t("history.open");

      const body = document.createElement("div");
      body.className = "history-body";
      const source = document.createElement("div");
      source.className = "history-source";
      source.textContent = result.sourceText || "";
      const translation = document.createElement("div");
      translation.className = "history-translation";
      translation.textContent = result.translation || "";
      const meta = document.createElement("div");
      meta.className = "history-meta";
      meta.textContent = [
        Glossy.i18n.providerName(result.provider),
        new Date(Number(entry.at) * 1000).toLocaleString(),
      ].join(" · ");
      body.append(source, translation, meta);

      const actions = document.createElement("div");
      actions.className = "history-actions";
      actions.append(
        historyButton("history.copy", COPY_ICON, async () => {
          await Glossy.invoke("copy_text", { text: String(result.translation || "") }).catch(
            () => false,
          );
          showToast(Glossy.i18n.t("toast.copied"));
        }),
        historyButton("history.remove", REMOVE_ICON, async () => {
          await Glossy.invoke("history_remove", { id: entry.id }).catch(() => null);
          await loadHistory();
        }),
      );

      const open = () => Glossy.invoke("history_reopen", { id: entry.id }).catch(() => null);
      item.addEventListener("click", open);
      item.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          open();
        }
      });

      item.append(body, actions);
      els.historyList.appendChild(item);
    });

    els.historyList.hidden = entries.length === 0;
    els.historyCount.textContent = history.length
      ? Glossy.i18n.t(
          entries.length === history.length ? "history.count" : "history.matching",
          entries.length,
          history.length,
        )
      : Glossy.i18n.t("history.empty");
  }

  async function clearHistory() {
    await Glossy.invoke("history_clear").catch(() => null);
    await loadHistory();
    showToast(Glossy.i18n.t("history.cleared"));
  }

  /** Writes the settings to Documents and reports where they landed. */
  async function exportSettings() {
    try {
      const path = await Glossy.invoke("export_settings");
      showToast(Glossy.i18n.t("backup.exported", path));
    } catch (error) {
      showToast(Glossy.i18n.t("backup.exportFailed") + Glossy.errorMessage(error));
    }
  }

  /** Replaces the settings with those of a file the user picked. */
  async function importSettings(file) {
    try {
      const json = await file.text();
      apply(await Glossy.invoke("import_settings", { json }));
      await refreshStatus();
      await loadHistory();
      showToast(Glossy.i18n.t("backup.imported"));
    } catch (error) {
      showToast(Glossy.i18n.t("backup.importFailed") + Glossy.errorMessage(error));
    }
  }

  /** Whether this build can update at all; false without a signing key. */
  let canUpdate = true;

  /** The release waiting to be installed, if any. */
  let pendingUpdate = null;

  /** Shows the line under the update buttons, empty when there is nothing to say. */
  function setUpdateStatus(key, ...args) {
    els.updateStatus.textContent = key ? Glossy.i18n.t(key, ...args) : "";
  }

  function rememberUpdate(info) {
    pendingUpdate = info || null;
    els.updateInstall.hidden = !pendingUpdate;
    els.updateCheck.hidden = !!pendingUpdate;
    if (pendingUpdate) setUpdateStatus("update.available", pendingUpdate.version);
    return pendingUpdate;
  }

  /** Asks the release feed; only a click on the button reports "up to date". */
  async function checkForUpdate(announce) {
    if (!canUpdate) return;
    els.updateCheck.disabled = true;
    try {
      const info = rememberUpdate(await Glossy.invoke("check_for_update"));
      if (info) showToast(Glossy.i18n.t("update.found", info.version));
      else if (announce) showToast(Glossy.i18n.t("update.upToDate"));
    } catch (error) {
      if (announce) showToast(Glossy.i18n.t("update.failed") + Glossy.errorMessage(error));
    } finally {
      els.updateCheck.disabled = false;
    }
  }

  async function installUpdate() {
    els.updateInstall.disabled = true;
    setUpdateStatus("update.downloading");
    try {
      // The new version only runs after a restart, so this call does not return
      // before Glossy has closed.
      await Glossy.invoke("install_update");
    } catch (error) {
      els.updateInstall.disabled = false;
      setUpdateStatus("update.available", pendingUpdate ? pendingUpdate.version : "");
      showToast(Glossy.i18n.t("update.failed") + Glossy.errorMessage(error));
    }
  }

  /** Hides the whole section on a build that cannot update itself. */
  async function loadUpdateCapability() {
    try {
      canUpdate = !!(await Glossy.invoke("update_capability"));
    } catch {
      canUpdate = false;
    }
    els.updateUnavailable.hidden = canUpdate;
    els.updateCheck.hidden = !canUpdate;
    els.checkUpdates.disabled = !canUpdate;
  }

  /** Fills the dropdown with the programs that currently own a visible window. */
  async function loadRunningApps() {
    const loading = new Option(Glossy.i18n.t("ignored.loading"), "");
    els.ignoredRunning.innerHTML = "";
    els.ignoredRunning.appendChild(loading);
    let apps = [];
    try {
      apps = (await Glossy.invoke("running_apps")) || [];
    } catch (error) {
      apps = [];
    }
    runningApps = apps.length;
    els.ignoredRunning.innerHTML = "";
    els.ignoredRunning.appendChild(
      new Option(
        apps.length ? Glossy.i18n.t("ignored.running") : Glossy.i18n.t("ignored.none"),
        "",
      ),
    );
    apps.forEach((app) => {
      const title = String(app.title || "").trim();
      const label = title ? app.name + " · " + (title.length > 48 ? title.slice(0, 48) + "…" : title) : app.name;
      els.ignoredRunning.appendChild(new Option(label, app.name));
    });
  }

  /** Re-labels the dropdown header after a language change; -1 means "loading". */
  function relabelRunningApps() {
    const head = els.ignoredRunning.options[0];
    if (!head || head.value !== "") return;
    const key = runningApps < 0 ? "ignored.loading" : runningApps ? "ignored.running" : "ignored.none";
    head.textContent = Glossy.i18n.t(key);
  }

  /** Re-applies the interface language to everything the script writes itself. */
  function applyLanguage() {
    Glossy.i18n.apply(document);
    renderIgnored();
    renderSourceLangs();
    relabelRunningApps();
    syncService();
    showHotkey(lastStatus);
    fillSample();
    syncDemoLanguage();
    renderHistory();
    if (settings) {
      // A pick that is not saved yet keeps its place; one the engine does not
      // translate settles on the first language it does.
      els.targetLang.value = fillLanguages(els.targetLang.value || settings.targetLang);
    }
  }

  /** Re-labels the translation card, and rebuilds the language bar with it. */
  function syncDemoLanguage() {
    els.demoCopy.setAttribute("aria-label", Glossy.i18n.t("popup.copy"));
    els.demoFrom.setAttribute("aria-label", Glossy.i18n.t("popup.sourceLang"));
    els.demoTo.setAttribute("aria-label", Glossy.i18n.t("popup.targetLang"));
    els.demoSwap.setAttribute("aria-label", Glossy.i18n.t("popup.swap"));
    if (!els.demoLangbar.hidden) showLanguages(demoResult);
  }

  function apply(next) {
    settings = next;
    els.enabled.checked = !!next.enabled;
    els.triggerOnDrag.checked = !!next.triggerOnDrag;
    els.triggerOnDoubleClick.checked = !!next.triggerOnDoubleClick;
    els.restoreClipboard.checked = !!next.restoreClipboard;
    els.showOriginal.checked = !!next.showOriginal;
    els.autostart.checked = !!next.autostart;
    els.minSelectionLen.value = String(numberOr(next.minSelectionLen, 2));
    ignored = normalizeIgnored(next.ignoredApps);
    sourceLangs = normalizeSourceLangs(next.sourceLangs);
    els.hotkey.value = next.hotkey || "";
    els.theme.value = themeOr(next.theme);
    els.fontScale.value = String(pick(FONT_SCALES, next.fontScale, 100));
    els.popupWidth.value = String(pick(POPUP_SIZES, next.popupWidth, 356));
    els.popupOpacity.value = String(pick(OPACITIES, next.popupOpacity, 100));
    els.autoCloseSecs.value = String(pick(AUTO_CLOSE, next.autoCloseSecs, 0));
    els.closeAfterCopy.checked = !!next.closeAfterCopy;
    els.unitsEnabled.checked = next.unitsEnabled !== false;
    els.demoUnits.checked = els.unitsEnabled.checked;
    els.wordSentence.checked = !!next.wordSentence;
    els.sentencePairs.checked = !!next.sentencePairs;
    els.compactPopup.checked = !!next.compactPopup;
    els.speechRate.value = String(pick(SPEECH_RATES, next.speechRate, 0));
    els.fallbackEnabled.checked = next.fallbackEnabled !== false;
    fallbackOrder = normalizeFallbackOrder(next.fallbackOrder);
    els.historyLimit.value = String(pick(HISTORY_LIMITS, next.historyLimit, 50));
    els.checkUpdates.checked = !!next.checkUpdates;
    applyTheme(els.theme.value);
    // The engine comes first: the two language lists are the languages it
    // takes, so what they offer depends on it.
    els.service.value = serviceOf(next);
    els.targetLang.value = fillLanguages(next.targetLang);
    els.uiLang.value = next.uiLang === "zh" || next.uiLang === "en" ? next.uiLang : "system";
    Glossy.i18n.set(els.uiLang.value);
    els.options.dataset.disabled = String(!next.enabled);
    applyLanguage();
    syncChannelQuota();
  }

  function collect() {
    const service = els.service.value;
    return {
      ...settings,
      enabled: els.enabled.checked,
      triggerOnDrag: els.triggerOnDrag.checked,
      triggerOnDoubleClick: els.triggerOnDoubleClick.checked,
      restoreClipboard: els.restoreClipboard.checked,
      showOriginal: els.showOriginal.checked,
      autostart: els.autostart.checked,
      minSelectionLen: numberOr(els.minSelectionLen.value, 2),
      ignoredApps: ignored,
      sourceLangs: sourceLangs,
      hotkey: els.hotkey.value.trim(),
      theme: els.theme.value,
      fontScale: numberOr(els.fontScale.value, 100),
      popupWidth: numberOr(els.popupWidth.value, 356),
      popupOpacity: numberOr(els.popupOpacity.value, 100),
      autoCloseSecs: numberOr(els.autoCloseSecs.value, 0),
      closeAfterCopy: els.closeAfterCopy.checked,
      // The stored shape still separates where the text goes from which engine
      // translates it: the two engines that go through Glossy's server are the
      // cloud channel, while the free Google endpoint dials out from here.
      channel: service === "google" ? "api" : "cloud",
      cloudProvider: "builtin",
      cloudVendor: CLOUD_VENDORS[service] || "",
      unitsEnabled: els.unitsEnabled.checked,
      wordSentence: els.wordSentence.checked,
      sentencePairs: els.sentencePairs.checked,
      compactPopup: els.compactPopup.checked,
      speechRate: numberOr(els.speechRate.value, 0),
      fallbackEnabled: els.fallbackEnabled.checked,
      fallbackOrder: fallbackOrder.slice(),
      historyLimit: numberOr(els.historyLimit.value, 50),
      checkUpdates: els.checkUpdates.checked,
      targetLang: els.targetLang.value,
      provider: service === "google" ? "google" : "baidu",
      uiLang: els.uiLang.value,
    };
  }

  async function save() {
    try {
      apply(await Glossy.invoke("save_settings", { settings: collect() }));
      showToast(Glossy.i18n.t("toast.saved"));
      await refreshStatus();
      // A change of the cap, or of nothing at all: the list stays right by
      // asking the backend what it remembers.
      await loadHistory();
    } catch (error) {
      showToast(Glossy.i18n.t("toast.saveFailed") + Glossy.errorMessage(error));
    }
  }

  function scheduleSave() {
    pending = true;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(flushSave, 180);
  }

  /**
   * Writes a change the debounce is still holding.
   *
   * The settings window is closed (and the window is hidden rather than
   * destroyed) as soon as the user is done, which happens well inside the 180ms
   * the debounce waits, so the last edit would be dropped without this.
   */
  function flushSave() {
    clearTimeout(saveTimer);
    saveTimer = 0;
    if (!pending) return;
    pending = false;
    save();
  }

  async function refreshStatus() {
    let status = null;
    try {
      status = await Glossy.invoke("capture_status");
    } catch (error) {
      setStatus("bad", Glossy.i18n.t("status.unavailable"), Glossy.errorMessage(error));
      return;
    }
    lastStatus = status;
    showHotkey(status);
    if (!els.enabled.checked) {
      setStatus("off", Glossy.i18n.t("status.paused"), Glossy.i18n.t("status.paused.title"));
    } else if (status && status.hooked) {
      setStatus("ok", Glossy.i18n.t("status.listening"), Glossy.i18n.t("status.listening.title"));
    } else {
      setStatus(
        "bad",
        Glossy.i18n.t("status.hook"),
        (status && status.error) || Glossy.i18n.t("status.hook.title"),
      );
    }
  }

  function showHotkey(status) {
    if (status && status.hotkey) {
      els.hotkeyHint.dataset.tone = "ok";
      els.hotkeyHint.textContent = Glossy.i18n.t("hotkey.active", status.hotkey);
    } else if (status && status.hotkeyError) {
      els.hotkeyHint.dataset.tone = "bad";
      els.hotkeyHint.textContent = status.hotkeyError;
    } else {
      delete els.hotkeyHint.dataset.tone;
      els.hotkeyHint.textContent = Glossy.i18n.t("hotkey.none");
    }
  }

  async function runDemo(value) {
    const text = String(value || "").trim();
    if (text.length < 2) return;

    demoValue = text;
    demoResult = null;
    const mine = ++demoTicket;
    els.demoCard.hidden = false;
    els.demoHeadword.textContent = text;
    els.demoLangbar.hidden = true;
    Glossy.render.loading(els.demoResult);
    try {
      const result = await Glossy.invoke("translate_text", {
        text,
        sourceLang: demoPair.source === AUTO ? null : demoPair.source,
        targetLang: demoPair.target,
      });
      if (mine !== demoTicket) return;
      demoResult = result;
      demoDetected = result.sourceLang || "";
      Glossy.render.result(els.demoResult, result, {
        showOriginal: els.showOriginal.checked,
        compact: els.compactPopup.checked,
      });
      if (result.sourceText) els.demoHeadword.textContent = String(result.sourceText);
      showLanguages(result);
      refineDemo(result, mine);
    } catch (error) {
      if (mine !== demoTicket) return;
      Glossy.render.error(els.demoResult, Glossy.errorMessage(error), () => runDemo(text));
    }
    // A translation through the shared server eats into the allowance shown
    // under the address, and a refusal is exactly when it is worth looking at,
    // so what is left is read again either way.
    if (cloudQuotaVisible() && mine === demoTicket) refreshCloudQuota();
  }

  /** Grows the demo card into the full word entry once the lookups have
      answered, the same way the floating popup does. */
  async function refineDemo(result, mine) {
    if (!result || result.kind !== "word") return;
    if (result.phonetic && (result.meanings || []).length && result.example) return;

    const waiting = { ...result, phonetic: null, meanings: [], example: null };
    Glossy.render.result(els.demoResult, waiting, {
      showOriginal: els.showOriginal.checked,
      compact: els.compactPopup.checked,
      pending: true,
    });

    let details = null;
    try {
      details = await Glossy.invoke("word_details", {
        text: result.sourceText || text,
        sourceLang: demoPair.source === AUTO ? null : demoPair.source,
        targetLang: result.targetLang || null,
      });
    } catch (error) {
      details = null;
    }
    if (mine !== demoTicket || !demoResult) return;

    demoResult = {
      ...demoResult,
      phonetic: demoResult.phonetic || (details && details.phonetic) || null,
      meanings: (demoResult.meanings || []).length
        ? demoResult.meanings
        : (details && details.meanings) || [],
      example: demoResult.example || (details && details.example) || null,
      synonyms: (demoResult.synonyms || []).length
        ? demoResult.synonyms
        : (details && details.synonyms) || [],
      forms: (demoResult.forms || []).length ? demoResult.forms : (details && details.forms) || [],
      context: demoResult.context || (details && details.context) || null,
    };
    // Drawn either way, so the card loses its placeholder when the lookups have
    // nothing to add.
    drawDemo();
  }

  /** Draws the sample card, with the switches of the popup applied to it. */
  function drawDemo() {
    Glossy.render.result(els.demoResult, demoResult, {
      showOriginal: els.showOriginal.checked,
      compact: els.compactPopup.checked,
    });
  }

  async function copyDemo() {
    const value = demoResult && demoResult.translation ? String(demoResult.translation) : "";
    if (!value) return;
    await Glossy.invoke("copy_text", { text: value }).catch(() => false);
    els.demoCopy.innerHTML = DONE_ICON;
    els.demoCopy.classList.add("done");
    clearTimeout(demoCopyTimer);
    demoCopyTimer = setTimeout(() => {
      els.demoCopy.innerHTML = COPY_ICON;
      els.demoCopy.classList.remove("done");
    }, 1100);
  }

  /** Label of the entry that hands the source language over to the provider. */
  function autoLabel() {
    const name = Glossy.languageName(demoDetected);
    const label = Glossy.i18n.t("popup.autoDetected");
    return name ? `${label} · ${name}` : label;
  }

  /**
   * Rebuilds one language dropdown from the languages the chosen engine
   * translates, and settles on the wanted value while it is one of them: on
   * "detect it" (the source bar) or on the first language the engine takes (the
   * target bar) otherwise.
   *
   * A code the shared list never had (a language the provider detected) is kept
   * as an extra entry, because it did not come from the menu.
   */
  function fillLanguageSelect(select, selected, withAuto) {
    const codes = Glossy.languagesFor(els.service.value);
    if (
      selected &&
      selected !== AUTO &&
      codes.indexOf(selected) === -1 &&
      LANGUAGES.indexOf(selected) === -1
    ) {
      codes.unshift(selected);
    }

    select.innerHTML = "";
    if (withAuto) {
      const option = document.createElement("option");
      option.value = AUTO;
      option.textContent = autoLabel();
      select.appendChild(option);
    }
    codes.forEach((code) => {
      const option = document.createElement("option");
      option.value = code;
      option.textContent = Glossy.languageName(code);
      select.appendChild(option);
    });
    const wanted = selected && codes.indexOf(selected) !== -1;
    select.value = wanted ? selected : withAuto ? AUTO : codes[0];
  }

  /** Reflects the pair of the current result in the language bar. */
  function showLanguages(result) {
    const target = demoPair.target || (result && result.targetLang) || "";
    if (!target) {
      els.demoLangbar.hidden = true;
      return;
    }
    fillLanguageSelect(els.demoFrom, demoPair.source, true);
    fillLanguageSelect(els.demoTo, target, false);
    els.demoLangbar.hidden = false;
  }

  /** Back to "detect the source and use the configured target". */
  function resetDemoLanguages() {
    demoPair = { source: AUTO, target: null };
    demoDetected = "";
  }

  /** Drops the demo languages the chosen engine does not translate, the same
      way the popup's bar does when the engine is switched under it. */
  function dropUnservedPair() {
    const unserved = (code) =>
      !!code && code !== AUTO && !Glossy.servesLanguage(els.service.value, code);
    if (unserved(demoPair.source)) demoPair.source = AUTO;
    if (unserved(demoPair.target)) demoPair.target = null;
  }

  /**
   * Remembers the language the card is translated into, so the next selection
   * starts from it rather than from the target the settings hold.
   */
  function rememberTarget(code) {
    if (!code || code === AUTO) return;
    Glossy.invoke("set_target_lang", { code }).catch((error) => {
      console.warn("glossy: could not remember the target language", error);
    });
  }

  /** Reverse-translates the card: the translation becomes the new selection. */
  function swapDemo() {
    if (!demoResult || !demoResult.translation) return;
    const value = String(demoResult.translation).trim();
    if (value.length < 2) return;

    const source = demoPair.source === AUTO ? demoResult.sourceLang : demoPair.source;
    const target = demoPair.target || demoResult.targetLang;
    demoPair = { source: target || AUTO, target: source || null };
    demoDetected = "";
    runDemo(value);
  }

  /** Text the user highlighted, in the box or anywhere else on the page. */
  function highlighted() {
    const onPage = String(window.getSelection() || "").trim();
    if (onPage) return onPage;
    const start = els.demoText.selectionStart;
    const end = els.demoText.selectionEnd;
    if (typeof start === "number" && typeof end === "number" && end > start) {
      return els.demoText.value.slice(start, end);
    }
    return "";
  }

  function translateHighlighted() {
    const picked = highlighted().replace(/\s+/g, " ").trim();
    if (picked.length < 2) return;
    selection = picked;
    els.selectionHint.textContent = picked.length > 42 ? picked.slice(0, 42) + "…" : picked;
    // A brand new selection starts from the configured languages again, exactly
    // like the popup does for every captured selection.
    resetDemoLanguages();
    runDemo(picked);
  }

  /** Keeps the sample text in the box while the user has not typed anything. */
  function fillSample() {
    const next = Glossy.i18n.t("demo.demoText");
    if (!els.demoText.value || els.demoText.value === sampleText) els.demoText.value = next;
    sampleText = next;
  }

  /** Draws the card again, so a units or a language change lands on the spot. */
  function refreshDemo() {
    if (demoValue) runDemo(demoValue);
  }

  /**
   * The translate area carries its own units switch. Both switches write the one
   * setting, so each mirrors the other. The backend attaches the conversions
   * from the settings it holds, so the write is awaited before the card is
   * redrawn — the conversions of the old switch would otherwise come back.
   */
  async function setUnits(enabled) {
    els.unitsEnabled.checked = enabled;
    els.demoUnits.checked = enabled;
    clearTimeout(saveTimer);
    saveTimer = 0;
    pending = false;
    await save();
    refreshDemo();
  }

  /** Puts the clipboard in the box and translates it right away. */
  async function pasteDemo() {
    let text = "";
    try {
      text = String((await Glossy.invoke("read_clipboard")) || "");
    } catch (error) {
      text = "";
    }
    if (!text.trim()) {
      showToast(Glossy.i18n.t("demo.pasteEmpty"));
      return;
    }
    els.demoText.value = text;
    resetDemoLanguages();
    runDemo(text);
  }

  /** Empties the box and takes the card away. */
  function clearDemo() {
    // A translation still in flight must not draw into the emptied card.
    demoTicket += 1;
    demoValue = "";
    demoResult = null;
    demoDetected = "";
    selection = "";
    resetDemoLanguages();
    els.demoText.value = "";
    els.demoCard.hidden = true;
    els.demoHeadword.textContent = "";
    els.demoResult.innerHTML = "";
    els.demoLangbar.hidden = true;
    els.selectionHint.textContent = Glossy.i18n.t("demo.nothing");
  }

  els.options.addEventListener("change", scheduleSave);
  els.enabled.addEventListener("change", () => {
    els.options.dataset.disabled = String(!els.enabled.checked);
    scheduleSave();
  });
  els.uiLang.addEventListener("change", () => {
    Glossy.i18n.set(els.uiLang.value);
    applyLanguage();
    scheduleSave();
  });
  els.service.addEventListener("change", async () => {
    syncService();
    syncChannelQuota();
    // What the two language lists offer follows the engine, and so does the
    // answer of the card; the save is awaited because the backend only knows
    // the new engine once it has landed.
    els.targetLang.value = fillLanguages(els.targetLang.value);
    dropUnservedPair();
    clearTimeout(saveTimer);
    saveTimer = 0;
    pending = false;
    await save();
    refreshDemo();
  });
  els.cloudQuotaRefresh.addEventListener("click", () => refreshCloudQuota());
  els.ignoredRunning.addEventListener("change", () => {
    const picked = els.ignoredRunning.value;
    els.ignoredRunning.value = "";
    if (!picked) return;
    addIgnored(picked);
    renderIgnored();
    scheduleSave();
  });
  els.ignoredAdd.addEventListener("click", () => {
    const value = els.ignoredInput.value.trim();
    if (!value) return;
    addIgnored(value);
    els.ignoredInput.value = "";
    renderIgnored();
    scheduleSave();
  });
  els.ignoredInput.addEventListener("keydown", (event) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    els.ignoredAdd.click();
  });
  els.sourceLangsAdd.addEventListener("change", () => {
    const code = els.sourceLangsAdd.value;
    els.sourceLangsAdd.value = "";
    if (!addSourceLang(code)) return;
    renderSourceLangs();
    scheduleSave();
  });
  els.sourceLangsClear.addEventListener("click", () => {
    if (!sourceLangs.length) return;
    sourceLangs = [];
    renderSourceLangs();
    scheduleSave();
  });
  els.ignoredPick.addEventListener("click", async () => {
    try {
      await Glossy.invoke("pick_app");
      showToast(Glossy.i18n.t("ignored.picking"));
    } catch (error) {
      showToast(Glossy.errorMessage(error));
    }
  });
  Glossy.listen("glossy://picked-app", (event) => {
    const payload = (event && event.payload) || {};
    const name = payload.name || "";
    if (!name) {
      showToast(Glossy.i18n.t("ignored.notFound"));
      return;
    }
    addIgnored(name);
    renderIgnored();
    scheduleSave();
    showToast(Glossy.i18n.t("ignored.picked", name));
  });
  els.theme.addEventListener("change", () => {
    applyTheme(els.theme.value);
    scheduleSave();
  });
  [
    els.targetLang,
    els.restoreClipboard,
    els.showOriginal,
    els.autostart,
    els.minSelectionLen,
    els.hotkey,
    els.fontScale,
    els.popupWidth,
    els.popupOpacity,
    els.autoCloseSecs,
    els.closeAfterCopy,
    els.unitsEnabled,
    els.wordSentence,
    els.sentencePairs,
    els.compactPopup,
    els.speechRate,
    els.fallbackEnabled,
    els.historyLimit,
    els.checkUpdates,
  ].forEach((element) => {
    element.addEventListener("change", scheduleSave);
  });
  // Registered after the loop, so the mirror of the area's switch runs with it.
  els.unitsEnabled.addEventListener("change", () => setUnits(els.unitsEnabled.checked));
  els.fallbackEnabled.addEventListener("change", renderFallbackOrder);
  els.compactPopup.addEventListener("change", () => {
    if (demoResult) drawDemo();
  });

  els.historySearch.addEventListener("input", renderHistory);
  els.historyClear.addEventListener("click", clearHistory);

  els.updateCheck.addEventListener("click", () => checkForUpdate(true));
  els.updateInstall.addEventListener("click", installUpdate);

  els.settingsExport.addEventListener("click", () => exportSettings());
  els.settingsImport.addEventListener("click", () => els.settingsFile.click());
  els.settingsFile.addEventListener("change", () => {
    const file = els.settingsFile.files && els.settingsFile.files[0];
    // Clearing the input lets the same file be picked twice in a row.
    els.settingsFile.value = "";
    if (file) importSettings(file);
  });

  els.demoText.addEventListener("mouseup", () => setTimeout(translateHighlighted, 0));
  els.demoText.addEventListener("keyup", (event) => {
    // A selection made with the keyboard (Shift+arrows, Ctrl+A) translates too.
    if (event.shiftKey || event.key === "a" || event.key === "A") translateHighlighted();
  });
  els.demoRun.addEventListener("click", () => {
    const picked = highlighted().replace(/\s+/g, " ").trim();
    runDemo(picked.length >= 2 ? picked : els.demoText.value);
  });
  els.demoPaste.addEventListener("click", pasteDemo);
  els.demoClear.addEventListener("click", clearDemo);
  els.demoText.addEventListener("keydown", (event) => {
    if (event.key !== "Enter" || !(event.ctrlKey || event.metaKey)) return;
    event.preventDefault();
    runDemo(els.demoText.value);
  });
  els.demoText.addEventListener("paste", () => {
    // The new text lands after the event, so the run waits a beat for it.
    setTimeout(() => {
      const value = els.demoText.value.trim();
      if (value.length < 2) return;
      resetDemoLanguages();
      runDemo(value);
    }, 0);
  });
  els.demoUnits.addEventListener("change", () => setUnits(els.demoUnits.checked));
  els.demoFrom.addEventListener("change", () => {
    demoPair.source = els.demoFrom.value;
    demoDetected = "";
    runDemo(demoValue);
  });
  els.demoTo.addEventListener("change", () => {
    demoPair.target = els.demoTo.value;
    rememberTarget(demoPair.target);
    runDemo(demoValue);
  });
  els.demoSwap.addEventListener("click", swapDemo);
  els.demoCopy.addEventListener("click", copyDemo);

  els.demoPopup.addEventListener("click", () => {
    const value = selection || els.demoText.value.replace(/\s+/g, " ").trim();
    if (value.length < 2) return;
    Glossy.invoke("show_popup", { text: value }).catch(() => {});
  });

  // A pending change is written before the window goes away, whichever way it
  // goes away: hidden by the close button, minimized, or unloaded.
  // A translation made anywhere else — the popup, the hotkey — lands in the
  // history while this window may be open, so the list reloads on the signal
  // instead of waiting for the next start.
  Glossy.listen("glossy://history", loadHistory);
  Glossy.listen("glossy://flush-settings", flushSave);
  // The card in the popup switches its own engine and picks its own target
  // language, and both are stored, so this window follows them — the fields it
  // does not own are left alone rather than saved back over the card's choice.
  Glossy.listen("glossy://settings", (event) => {
    const next = (event && event.payload) || null;
    if (!next || !settings) return;
    const service = serviceOf(next);
    const language = String(next.targetLang || settings.targetLang || "");
    const switched = els.service.value !== service;
    settings = {
      ...settings,
      targetLang: language,
      channel: next.channel,
      cloudProvider: next.cloudProvider,
      cloudVendor: next.cloudVendor,
      provider: next.provider,
    };
    if (switched) {
      els.service.value = service;
      syncService();
      syncChannelQuota();
      dropUnservedPair();
    }
    if (els.targetLang.value !== language) els.targetLang.value = fillLanguages(language);
    if (switched) refreshDemo();
  });
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) flushSave();
  });
  window.addEventListener("blur", flushSave);
  window.addEventListener("pagehide", flushSave);
  window.addEventListener("beforeunload", flushSave);

  (async function start() {
    // Resolved before the settings paint the window, so the effect is in place
    // from the first frame on.
    await resolveBackdrop();
    // Which languages an engine translates is asked for first, so the lists
    // already leave out what the chosen engine cannot do.
    try {
      Glossy.setServiceLanguages(await Glossy.invoke("service_languages"));
    } catch (error) {
      console.warn("glossy: cannot read the language tables", error);
    }
    try {
      apply(await Glossy.readSettings());
    } catch (error) {
      showToast(Glossy.i18n.t("toast.loadFailed") + Glossy.errorMessage(error));
    }
    await refreshStatus();
    Glossy.listen("glossy://status", refreshStatus);
    setInterval(refreshStatus, 4000);
    loadRunningApps();
    await loadHistory();
    await loadUpdateCapability();
    // The start-up check ran before this window existed, so it arrives as an
    // event rather than as the answer to a call.
    Glossy.listen("glossy://update", (event) => {
      rememberUpdate((event && event.payload) || null);
    });
  })();
})(window.Glossy);

(() => {
  const { invoke } = window.__TAURI__.core;
  const { listen } = window.__TAURI__.event;

  const MESSAGES = {
    en: {
      app_title: 'Clauwitch',
      header_title: 'Claude accounts',
      header_subtitle: 'Each account is kept as its own session. Switching reopens Claude with your saved sign-in.',
      default_account: 'Original account',
      language_tooltip: 'Change language',
      window_minimize: 'Minimize window',
      window_maximize: 'Maximize window',
      window_restore: 'Restore window',
      window_close: 'Close window',
      status_open: 'Open',
      status_closed: 'Closed',
      status_saved: 'Saved',
      active: 'Active',
      last_used: 'Last used: {0}',
      not_used: 'Not used yet',
      add_account: 'Add account',
      switch: 'Switch',
      open: 'Open',
      bring_front: 'Bring to front',
      rename: 'Rename',
      close: 'Close',
      delete: 'Delete',
      open_folder: 'Open folder',
      cancel: 'Cancel',
      ok: 'OK',
      continue: 'Continue',
      more_actions: 'More actions',
      menu_open: 'Open',
      menu_switch: 'Switch to this account',
      menu_bring_front: 'Bring to front',
      menu_rename: 'Rename',
      menu_close: 'Close Claude',
      menu_delete: 'Delete account',
      menu_open_folder: 'Open folder',
      add_title: 'New account',
      add_prompt: 'Enter a name for this account (for example Work or Personal):',
      add_confirm: 'The Claude window that is open now will be closed and a fresh session will open for the new account.\n\nContinue?',
      rename_title: 'Rename',
      rename_prompt: 'New name for this account:',
      delete_title: 'Delete account',
      delete_body: 'Delete “{0}”?\n\nThis removes the account\'s session and local data from this computer. Your Anthropic account is not deleted.',
      delete_active: 'This account is active. Deleting it will switch back to the Original account.',
      info_title: 'Information',
      error_title: 'Something went wrong',
      busy_add: 'Preparing the new account...',
      busy_switch: 'Switching to “{0}”...',
      busy_open: 'Opening Claude...',
      busy_bring: 'Bringing Claude to the front...',
      busy_close: 'Closing Claude...',
      busy_delete: 'Deleting “{0}”...',
      busy_update: 'Applying changes...',
      notice_added: 'New account opened. Sign in in the Claude window; the session will be saved to this account.',
      notice_switched: 'Switched to “{0}”.',
      notice_closed: 'Claude closed.',
      notice_deleted: '“{0}” was deleted.',
      footer_account_one: '1 account',
      footer_account_many: '{0} accounts',
      footer_active: 'Active: {0}',
      footer_open: 'Claude is open',
      footer_closed: 'Claude is closed',
      footer_foreign: 'another Claude window is open',
      empty_title: 'No other accounts yet',
      empty_body: 'Use “Add account” to add another Claude account. Each account signs in once and its session is saved.',
      banner_live_data_missing: 'Claude Desktop’s data folder could not be found. Open Claude once, then restart this app.',
      banner_launcher_missing: 'Claude Desktop could not be found. Make sure it is installed.',
      error_name_empty: 'The account name cannot be empty.',
      error_name_long: 'The account name can be at most 60 characters.',
      error_name_exists: 'An account with this name already exists.',
      error_live_data_missing: 'The Claude Desktop data folder could not be found. Open and close Claude Desktop once, then start this app again.',
      error_launcher_missing: 'Claude Desktop could not be found. Make sure it is installed.',
      error_close_failed: 'Claude could not be closed completely. Close its window and try again.',
      error_switch_failed: 'The account could not be switched. Please try again.',
      error_launch_failed: 'Claude could not be started.',
      error_delete_failed: 'The account could not be deleted. Make sure Claude is closed and try again.',
      error_folder_missing: 'This account does not have a local folder yet. Open it once with Claude, then try again.',
      error_folder_open_failed: 'The folder could not be opened.',
      error_unexpected: 'An unexpected error occurred.'
    },
    tr: {
      app_title: 'Clauwitch',
      header_title: 'Claude hesapları',
      header_subtitle: 'Her hesap kendi oturumuyla saklanır. Geçiş yaptığınızda Claude, kayıtlı girişinizle yeniden açılır.',
      default_account: 'Orijinal hesap',
      language_tooltip: 'Dili değiştir',
      window_minimize: 'Pencereyi küçült',
      window_maximize: 'Pencereyi büyüt',
      window_restore: 'Pencereyi geri yükle',
      window_close: 'Pencereyi kapat',
      status_open: 'Açık',
      status_closed: 'Kapalı',
      status_saved: 'Kayıtlı',
      active: 'Etkin',
      last_used: 'Son kullanım: {0}',
      not_used: 'Henüz kullanılmadı',
      add_account: 'Hesap ekle',
      switch: 'Geçiş yap',
      open: 'Aç',
      bring_front: 'Öne getir',
      rename: 'Yeniden adlandır',
      close: 'Kapat',
      delete: 'Sil',
      open_folder: 'Klasörü aç',
      cancel: 'İptal',
      ok: 'Tamam',
      continue: 'Devam et',
      more_actions: 'Diğer işlemler',
      menu_open: 'Aç',
      menu_switch: 'Bu hesaba geç',
      menu_bring_front: 'Öne getir',
      menu_rename: 'Yeniden adlandır',
      menu_close: 'Claude\'u kapat',
      menu_delete: 'Hesabı sil',
      menu_open_folder: 'Klasörü aç',
      add_title: 'Yeni hesap',
      add_prompt: 'Bu hesap için bir ad girin (örn. İş, Kişisel):',
      add_confirm: 'Yeni hesap için şu an açık olan Claude penceresi kapatılacak ve boş bir oturum açılacak.\n\nDevam edilsin mi?',
      rename_title: 'Yeniden adlandır',
      rename_prompt: 'Bu hesabın yeni adı:',
      delete_title: 'Hesabı sil',
      delete_body: '“{0}” hesabı silinsin mi?\n\nBu hesabın bu bilgisayardaki oturumu ve yerel verileri silinir. Anthropic hesabınız silinmez.',
      delete_active: 'Bu hesap şu an etkin. Silinirse Orijinal hesaba geçilir.',
      info_title: 'Bilgi',
      error_title: 'Bir sorun oluştu',
      busy_add: 'Yeni hesap hazırlanıyor...',
      busy_switch: '“{0}” hesabına geçiliyor...',
      busy_open: 'Claude açılıyor...',
      busy_bring: 'Claude öne getiriliyor...',
      busy_close: 'Claude kapatılıyor...',
      busy_delete: '“{0}” siliniyor...',
      busy_update: 'Değişiklikler uygulanıyor...',
      notice_added: 'Yeni hesap açıldı. Claude penceresinde giriş yapın; oturum bu hesaba kaydedilecek.',
      notice_switched: '“{0}” hesabına geçildi.',
      notice_closed: 'Claude kapatıldı.',
      notice_deleted: '“{0}” silindi.',
      footer_account_one: '1 hesap',
      footer_account_many: '{0} hesap',
      footer_active: 'Etkin: {0}',
      footer_open: 'Claude açık',
      footer_closed: 'Claude kapalı',
      footer_foreign: 'başka bir Claude penceresi açık',
      empty_title: 'Henüz ikinci bir hesap yok',
      empty_body: 'Yeni bir Claude hesabı eklemek için “Hesap ekle” düğmesini kullanın. Her hesap yalnızca bir kez giriş yapar; oturumu kaydedilir.',
      banner_live_data_missing: 'Claude Desktop veri klasörü bulunamadı. Claude\'u bir kez açıp kapatın, sonra uygulamayı yeniden başlatın.',
      banner_launcher_missing: 'Claude Desktop bulunamadı. Kurulu olduğundan emin olun.',
      error_name_empty: 'Hesap adı boş olamaz.',
      error_name_long: 'Hesap adı en fazla 60 karakter olabilir.',
      error_name_exists: 'Bu ada sahip bir hesap zaten var.',
      error_live_data_missing: 'Claude Desktop veri klasörü bulunamadı. Önce Claude Desktop\'ı açıp kapatın, sonra bu uygulamayı yeniden başlatın.',
      error_launcher_missing: 'Claude Desktop bulunamadı. Kurulu olduğundan emin olun.',
      error_close_failed: 'Claude tamamen kapatılamadı. Pencereyi kapatıp tekrar deneyin.',
      error_switch_failed: 'Hesap değiştirilemedi. Lütfen tekrar deneyin.',
      error_launch_failed: 'Claude başlatılamadı.',
      error_delete_failed: 'Hesap silinemedi. Claude\'un kapalı olduğundan emin olup tekrar deneyin.',
      error_folder_missing: 'Bu hesabın henüz yerel bir klasörü yok. Claude ile bir kez açtıktan sonra tekrar deneyin.',
      error_folder_open_failed: 'Klasör açılamadı.',
      error_unexpected: 'Beklenmeyen bir hata oluştu.'
    }
  };

  const AVATAR_COLORS = [
    '#788C5D', '#5B7FA6', '#8B82A6', '#5F8F86',
    '#B08A4F', '#A66A6A', '#7A7FA6', '#5E5D59'
  ];

  const state = {
    profiles: [],
    activeId: null,
    language: 'en',
    theme: 'light',
    isMaximized: false,
    anyRunning: false,
    activeRunning: false,
    livePath: null,
    launcherFound: true,
    selectedId: null,
    busy: false,
    busyMessage: '',
    notice: '',
    noticeUntil: 0,
    noticeSeq: 0,
    version: 0,
    loaded: false,
    lastSignature: '',
    pollBusy: false
  };

  const $ = id => document.getElementById(id);
  const els = {
    app: $('app'),
    card: document.querySelector('.card'),
    list: $('accountList'),
    empty: $('emptyState'),
    emptyTitle: $('emptyTitle'),
    emptyBody: $('emptyBody'),
    headerTitle: $('headerTitle'),
    headerSubtitle: $('headerSubtitle'),
    languageButton: $('languageButton'),
    languageLabel: $('languageLabel'),
    addButton: $('addButton'),
    addLabel: $('addLabel'),
    primaryButton: $('primaryButton'),
    primaryLabel: $('primaryLabel'),
    primaryIcon: $('primaryIcon'),
    status: $('statusText'),
    progress: $('progress'),
    banner: $('banner'),
    modalLayer: $('modalLayer'),
    windowControls: $('windowControls'),
    windowMinimize: $('windowMinimize'),
    windowMaximize: $('windowMaximize'),
    windowMaximizeIcon: $('windowMaximizeIcon'),
    windowClose: $('windowClose'),
    titlebarDragLayer: $('titlebarDragLayer')
  };

  function t(key, ...args) {
    const dictionary = MESSAGES[state.language] || MESSAGES.en;
    const fallback = MESSAGES.en;
    let template = dictionary[key] ?? fallback[key] ?? key;
    for (let index = 0; index < args.length; index += 1) {
      template = template.replaceAll(`{${index}}`, String(args[index]));
    }
    return template;
  }

  function escapeHtml(value) {
    return String(value ?? '').replace(/[&<>"']/g, character => ({
      '&': '&amp;',
      '<': '&lt;',
      '>': '&gt;',
      '"': '&quot;',
      "'": '&#39;'
    }[character]));
  }

  function initialFor(name) {
    const text = String(name ?? '').trim();
    return text ? text.slice(0, 1).toLocaleUpperCase(state.language === 'tr' ? 'tr-TR' : 'en-US') : '?';
  }

  function avatarColor(id) {
    let hash = 17;
    for (const character of String(id ?? '')) {
      hash = hash * 31 + character.charCodeAt(0);
    }
    return AVATAR_COLORS[(hash & 0x7fffffff) % AVATAR_COLORS.length];
  }

  function languageName(code) {
    return code === 'tr' ? 'Türkçe' : 'English';
  }

  function formatDate(milliseconds) {
    if (!milliseconds) {
      return t('not_used');
    }
    try {
      return new Intl.DateTimeFormat(state.language === 'tr' ? 'tr-TR' : 'en-US', {
        day: 'numeric',
        month: 'short',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      }).format(new Date(milliseconds));
    } catch {
      return t('not_used');
    }
  }

  function displayName(profile) {
    return profile.is_default ? t('default_account') : profile.name;
  }

  function profileById(id) {
    return state.profiles.find(profile => profile.id === id) || null;
  }

  function selectedProfile() {
    return profileById(state.selectedId) || profileById(state.activeId);
  }

  function activeProfile() {
    return profileById(state.activeId) || state.profiles[0] || null;
  }

  function canManage() {
    return Boolean(state.livePath && state.launcherFound);
  }

  function signature() {
    return [
      state.language,
      state.activeId,
      state.profiles.map(profile =>
        `${profile.id}:${profile.name}:${profile.is_default ? 1 : 0}:${profile.last_used_ms ?? 0}`
      ).join('|')
    ].join('~');
  }

  function applyState(dto, options = {}) {
    const languageChanged = state.loaded && dto.language && dto.language !== state.language;

    state.profiles = Array.isArray(dto.profiles) ? dto.profiles : [];
    state.activeId = dto.active_id;
    state.language = dto.language === 'tr' ? 'tr' : 'en';
    state.theme = dto.theme === 'dark' ? 'dark' : 'light';
    state.anyRunning = Boolean(dto.any_running);
    state.activeRunning = Boolean(dto.active_running);
    state.livePath = dto.live_path || null;
    state.launcherFound = Boolean(dto.launcher_found);

    if (options.selectActive) {
      state.selectedId = state.activeId;
    }
    if (!profileById(state.selectedId)) {
      state.selectedId = state.activeId || (state.profiles[0]?.id ?? null);
    }

    document.documentElement.dataset.theme = state.theme;

    if (!state.loaded || languageChanged) {
      applyStaticTranslations();
    }

    renderBanner();
    renderAccounts(options.forceRebuild || languageChanged);
    renderFooter();
    renderPrimary();
    state.loaded = true;
  }

  function applyStaticTranslations() {
    document.documentElement.lang = state.language;
    document.title = t('app_title');
    els.headerTitle.textContent = t('header_title');
    els.headerSubtitle.textContent = t('header_subtitle');
    els.languageLabel.textContent = languageName(state.language);
    els.addLabel.textContent = t('add_account');
    els.emptyTitle.textContent = t('empty_title');
    els.emptyBody.textContent = t('empty_body');
    els.languageButton.setAttribute('aria-label', t('language_tooltip'));
    els.languageButton.title = t('language_tooltip');
    updateWindowControls();
    renderBanner();
  }

  function updateWindowControls() {
    if (!els.windowMinimize) {
      return;
    }

    const maximizeLabel = state.isMaximized ? t('window_restore') : t('window_maximize');
    els.windowMinimize.setAttribute('aria-label', t('window_minimize'));
    els.windowMinimize.title = t('window_minimize');
    els.windowMaximize.setAttribute('aria-label', maximizeLabel);
    els.windowMaximize.title = maximizeLabel;
    els.windowMaximizeIcon?.setAttribute(
      'href',
      state.isMaximized ? '#icon-window-restore' : '#icon-window-maximize'
    );
    els.windowClose.setAttribute('aria-label', t('window_close'));
    els.windowClose.title = t('window_close');

    for (const button of [els.windowMinimize, els.windowMaximize, els.windowClose]) {
      button.disabled = state.busy;
    }
  }

  function setMaximized(maximized) {
    state.isMaximized = Boolean(maximized);
    els.app.classList.toggle('is-maximized', state.isMaximized);
    updateWindowControls();
  }

  async function refreshWindowState() {
    try {
      setMaximized(await invoke('window_is_maximized'));
    } catch {
    }
  }

  async function minimizeWindow() {
    if (state.busy) {
      return;
    }
    try {
      await invoke('window_minimize');
    } catch {
    }
  }

  async function toggleMaximize() {
    if (state.busy) {
      return;
    }
    try {
      setMaximized(await invoke('window_toggle_maximize'));
      window.setTimeout(refreshWindowState, 90);
    } catch {
      await refreshWindowState();
    }
  }

  async function closeWindow() {
    if (state.busy) {
      return;
    }
    try {
      await invoke('window_close');
    } catch {
    }
  }

  function renderBanner() {
    let message = '';
    if (!state.livePath) {
      message = t('banner_live_data_missing');
    } else if (!state.launcherFound) {
      message = t('banner_launcher_missing');
    }

    if (message) {
      els.banner.textContent = message;
      els.banner.hidden = false;
    } else {
      els.banner.hidden = true;
      els.banner.textContent = '';
    }
  }

  function renderAccounts(force = false) {
    const currentSignature = signature();
    const hasUserProfiles = state.profiles.some(profile => !profile.is_default);

    els.card.classList.toggle('is-empty', !hasUserProfiles);
    els.empty.hidden = hasUserProfiles;

    if (!force && currentSignature === state.lastSignature) {
      updateRowStatuses();
      return;
    }

    state.lastSignature = currentSignature;

    if (state.profiles.length === 0) {
      els.list.innerHTML = '';
      return;
    }

    const listHadFocus = Boolean(document.activeElement?.closest?.('.account-row'));

    els.list.innerHTML = state.profiles.map(profile => {
      const selected = profile.id === state.selectedId;
      const isActive = profile.id === state.activeId;
      const status = isActive
        ? (state.activeRunning ? 'open' : 'closed')
        : 'saved';
      const statusText = status === 'open'
        ? t('status_open')
        : status === 'closed' ? t('status_closed') : t('status_saved');
      const name = displayName(profile);
      const meta = profile.last_used_ms
        ? t('last_used', formatDate(profile.last_used_ms))
        : t('not_used');
      const avatar = profile.is_default
        ? '<img class="avatar avatar-icon" src="app-icon.png" alt="">'
        : `<div class="avatar" style="--avatar:${avatarColor(profile.id)}"><span>${escapeHtml(initialFor(name))}</span></div>`;
      const activeBadge = isActive
        ? `<span class="active-badge">${escapeHtml(t('active'))}</span>`
        : '';

      return `
        <div class="account-row${selected ? ' selected' : ''}" role="option"
             aria-selected="${selected ? 'true' : 'false'}"
             tabindex="${selected ? '0' : '-1'}"
             data-id="${escapeHtml(profile.id)}">
          ${avatar}
          <div class="account-main">
            <div class="account-name"><span class="name">${escapeHtml(name)}</span>${activeBadge}</div>
            <div class="account-meta">${escapeHtml(meta)}</div>
          </div>
          <span class="status-pill status-${status}"><span class="status-dot"></span>${escapeHtml(statusText)}</span>
          <button class="more-button" type="button" aria-label="${escapeHtml(t('more_actions'))}">
            <svg class="icon"><use href="#icon-more"/></svg>
          </button>
        </div>`;
    }).join('');

    updateRowStatuses();

    if (listHadFocus) {
      const selectedRow = els.list.querySelector('.account-row.selected');
      selectedRow?.focus({ preventScroll: true });
    }
  }

  function updateRowStatuses() {
    for (const profile of state.profiles) {
      const row = els.list.querySelector(`.account-row[data-id="${cssEscape(profile.id)}"]`);
      if (!row) {
        continue;
      }
      const isActive = profile.id === state.activeId;
      const status = isActive
        ? (state.activeRunning ? 'open' : 'closed')
        : 'saved';
      const pill = row.querySelector('.status-pill');
      if (!pill) {
        continue;
      }
      pill.className = `status-pill status-${status}`;
      const dot = pill.querySelector('.status-dot');
      if (dot) {
        pill.textContent = '';
        pill.appendChild(dot);
        pill.appendChild(document.createTextNode(
          status === 'open' ? t('status_open')
            : status === 'closed' ? t('status_closed') : t('status_saved')
        ));
      }
    }
  }

  function cssEscape(value) {
    if (window.CSS?.escape) {
      return window.CSS.escape(String(value));
    }
    return String(value).replace(/["\\]/g, '\\$&');
  }

  function renderFooter() {
    if (state.busy) {
      els.status.textContent = state.busyMessage;
      els.status.classList.remove('notice');
      return;
    }

    if (state.notice && Date.now() < state.noticeUntil) {
      els.status.textContent = state.notice;
      els.status.classList.add('notice');
      return;
    }

    state.notice = '';
    els.status.classList.remove('notice');

    const count = state.profiles.length;
    const accounts = count === 1 ? t('footer_account_one') : t('footer_account_many', count);
    const active = activeProfile();
    const activeName = active ? displayName(active) : '—';
    const running = state.anyRunning && !state.activeRunning
      ? t('footer_foreign')
      : state.anyRunning ? t('footer_open') : t('footer_closed');

    els.status.textContent = `${accounts}  ·  ${t('footer_active', activeName)}  ·  ${running}`;
  }

  function renderPrimary() {
    const profile = selectedProfile();
    const active = Boolean(profile && profile.id === state.activeId);

    if (!profile) {
      els.primaryLabel.textContent = t('open');
      els.primaryButton.disabled = true;
      return;
    }

    const primaryIcon = !active
      ? '#icon-swap'
      : state.activeRunning ? '#icon-bring' : '#icon-open';
    els.primaryIcon?.setAttribute('href', primaryIcon);

    if (!active) {
      els.primaryLabel.textContent = t('switch');
    } else if (state.activeRunning) {
      els.primaryLabel.textContent = t('bring_front');
    } else {
      els.primaryLabel.textContent = t('open');
    }

    els.primaryButton.disabled = state.busy || !canManage();
    els.addButton.disabled = state.busy || !canManage();
    els.languageButton.disabled = state.busy;
  }

  function selectAccount(id, options = {}) {
    if (!profileById(id)) {
      return;
    }

    state.selectedId = id;

    for (const row of els.list.querySelectorAll('.account-row')) {
      const selected = row.dataset.id === id;
      row.classList.toggle('selected', selected);
      row.setAttribute('aria-selected', selected ? 'true' : 'false');
      row.tabIndex = selected ? 0 : -1;
    }

    if (options.focus) {
      const row = els.list.querySelector('.account-row.selected');
      row?.focus({ preventScroll: true });
      row?.scrollIntoView({ block: 'nearest' });
    }

    renderPrimary();
    renderFooter();
  }

  function showNotice(text) {
    state.notice = text;
    state.noticeUntil = Date.now() + 9000;
    state.noticeSeq += 1;
    const sequence = state.noticeSeq;
    renderFooter();
    window.setTimeout(() => {
      if (sequence === state.noticeSeq) {
        state.notice = '';
        renderFooter();
      }
    }, 9000);
  }

  function setBusy(busy, message = '') {
    state.busy = busy;
    state.busyMessage = message;
    document.body.classList.toggle('busy', busy);
    els.progress.hidden = !busy;
    els.addButton.disabled = busy || !canManage();
    els.primaryButton.disabled = busy || !canManage();
    els.languageButton.disabled = busy;
    updateWindowControls();
    if (busy) {
      closeMenu();
      renderFooter();
    } else {
      renderFooter();
      renderPrimary();
    }
  }

  async function runCommand(command, args, busyMessage, successNotice, options = {}) {
    if (state.busy) {
      return null;
    }

    setBusy(true, busyMessage);
    state.version += 1;
    const version = state.version;

    try {
      const dto = await invoke(command, args);
      if (version === state.version) {
        applyState(dto, { selectActive: Boolean(options.selectActive), forceRebuild: true });
        if (successNotice) {
          showNotice(successNotice);
        }
      }
      return dto;
    } catch (error) {
      await refresh(true);
      showError(error);
      return null;
    } finally {
      setBusy(false);
    }
  }

  function errorCode(error) {
    const raw = String(error?.message ?? error ?? '');
    const keys = Object.keys(MESSAGES.en).filter(key => key.startsWith('error_'));
    return keys.find(key => raw.includes(key)) || 'error_unexpected';
  }

  function showError(error) {
    openModal({
      title: t('error_title'),
      message: t(errorCode(error)),
      confirmText: t('ok'),
      cancelText: null,
      input: false,
      danger: false
    });
  }

  async function refresh(silent = true) {
    if (state.busy || state.pollBusy) {
      return;
    }

    state.pollBusy = true;
    const version = state.version;

    try {
      const dto = await invoke('get_state');
      if (version === state.version && !state.busy) {
        applyState(dto);
      }
    } catch (error) {
      if (!silent || !state.loaded) {
        showError(error);
      }
    } finally {
      state.pollBusy = false;
    }
  }

  function nextAccountName() {
    let number = 2;
    while (number < 10_000) {
      const candidate = state.language === 'tr' ? `Hesap ${number}` : `Account ${number}`;
      const taken = state.profiles.some(profile =>
        !profile.is_default && profile.name.toLocaleLowerCase() === candidate.toLocaleLowerCase());
      if (!taken) {
        return candidate;
      }
      number += 1;
    }
    return state.language === 'tr' ? 'Hesap' : 'Account';
  }

  async function addAccount() {
    if (state.busy || !canManage()) {
      return;
    }

    const name = await openModal({
      title: t('add_title'),
      message: t('add_prompt'),
      confirmText: t('ok'),
      cancelText: t('cancel'),
      input: true,
      initial: nextAccountName(),
      danger: false,
      icon: 'check'
    });

    if (name === null) {
      return;
    }

    if (state.anyRunning) {
      const proceed = await openModal({
        title: t('add_title'),
        message: t('add_confirm'),
        confirmText: t('continue'),
        cancelText: t('cancel'),
        input: false,
        danger: false,
        icon: 'chevron-right'
      });
      if (!proceed) {
        return;
      }
    }

    await runCommand('add_account', { name }, t('busy_add'), t('notice_added'), {
      selectActive: true
    });
  }

  async function switchAccount(id) {
    const profile = profileById(id);
    if (!profile || state.busy) {
      return;
    }

    await runCommand(
      'switch_account',
      { id },
      t('busy_switch', displayName(profile)),
      t('notice_switched', displayName(profile)),
      { selectActive: true }
    );
  }

  async function openActive() {
    if (state.busy) {
      return;
    }
    await runCommand(
      'open_active',
      {},
      state.activeRunning ? t('busy_bring') : t('busy_open'),
      null
    );
  }

  function activateSelected() {
    const profile = selectedProfile();
    if (!profile || state.busy) {
      return;
    }
    if (profile.id === state.activeId) {
      openActive();
    } else {
      switchAccount(profile.id);
    }
  }

  async function renameSelected(profile) {
    if (!profile || profile.is_default || state.busy) {
      return;
    }

    const name = await openModal({
      title: t('rename_title'),
      message: t('rename_prompt'),
      confirmText: t('ok'),
      cancelText: t('cancel'),
      input: true,
      initial: profile.name,
      danger: false,
      icon: 'check'
    });

    if (name === null) {
      return;
    }

    try {
      const dto = await invoke('rename_account', { id: profile.id, name });
      applyState(dto, { forceRebuild: true });
    } catch (error) {
      showError(error);
    }
  }

  async function deleteSelected(profile) {
    if (!profile || profile.is_default || state.busy) {
      return;
    }

    let message = t('delete_body', displayName(profile));
    if (profile.id === state.activeId) {
      message += `\n\n${t('delete_active')}`;
    }

    const proceed = await openModal({
      title: t('delete_title'),
      message,
      confirmText: t('menu_delete'),
      cancelText: t('cancel'),
      input: false,
      danger: true,
      icon: 'trash'
    });

    if (!proceed) {
      return;
    }

    await runCommand(
      'delete_account',
      { id: profile.id },
      t('busy_delete', displayName(profile)),
      t('notice_deleted', displayName(profile)),
      { selectActive: true }
    );
  }

  async function closeClaude() {
    if (state.busy) {
      return;
    }
    await runCommand('close_claude', {}, t('busy_close'), t('notice_closed'));
  }

  async function openFolder(profile) {
    if (!profile || state.busy) {
      return;
    }
    try {
      await invoke('open_folder', { id: profile.id });
    } catch (error) {
      showError(error);
    }
  }

  async function setLanguage(language) {
    if (language === state.language) {
      closeMenu();
      return;
    }
    closeMenu();
    try {
      const dto = await invoke('set_language', { lang: language });
      applyState(dto, { forceRebuild: true });
    } catch (error) {
      showError(error);
    }
  }

  function openLanguageMenu() {
    if (state.busy) {
      return;
    }

    if (document.querySelector('.menu')) {
      closeMenu();
      return;
    }

    openMenu(els.languageButton, [
      {
        label: 'Türkçe',
        checked: state.language === 'tr',
        action: () => setLanguage('tr')
      },
      {
        label: 'English',
        checked: state.language === 'en',
        action: () => setLanguage('en')
      }
    ], true);

    els.languageButton.setAttribute('aria-expanded', 'true');
  }

  function openMoreMenu(profile, anchor) {
    const active = profile.id === state.activeId;
    const items = [
      {
        label: active
          ? (state.activeRunning ? t('menu_bring_front') : t('menu_open'))
          : t('menu_switch'),
        icon: 'swap',
        action: () => active ? openActive() : switchAccount(profile.id)
      }
    ];

    if (!profile.is_default) {
      items.push({
        label: t('menu_rename'),
        icon: 'pencil',
        action: () => renameSelected(profile)
      });
    }

    items.push({
      label: t('menu_open_folder'),
      icon: 'folder',
      action: () => openFolder(profile)
    });

    items.push({ separator: true });

    items.push({
      label: t('menu_close'),
      icon: 'power',
      disabled: !state.anyRunning,
      action: closeClaude
    });

    if (!profile.is_default) {
      items.push({
        label: t('menu_delete'),
        icon: 'trash',
        danger: true,
        action: () => deleteSelected(profile)
      });
    }

    openMenu(anchor, items, true);
  }

  let menuCleanup = null;

  function closeMenu() {
    document.querySelectorAll('.menu').forEach(menu => menu.remove());
    if (menuCleanup) {
      menuCleanup();
      menuCleanup = null;
    }
    els.languageButton.setAttribute('aria-expanded', 'false');
  }

  function openMenu(anchor, items, alignRight) {
    closeMenu();

    const menu = document.createElement('div');
    menu.className = 'menu';
    menu.setAttribute('role', 'menu');

    for (const item of items) {
      if (item.separator) {
        const separator = document.createElement('div');
        separator.className = 'menu-separator';
        menu.appendChild(separator);
        continue;
      }

      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'menu-item';
      button.setAttribute('role', 'menuitem');
      if (item.danger) {
        button.classList.add('danger');
      }
      button.disabled = Boolean(item.disabled);

      if (item.icon) {
        const icon = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        icon.setAttribute('class', 'icon');
        const use = document.createElementNS('http://www.w3.org/2000/svg', 'use');
        use.setAttribute('href', `#icon-${item.icon}`);
        icon.appendChild(use);
        button.appendChild(icon);
      }

      const label = document.createElement('span');
      label.className = 'menu-label';
      label.textContent = item.label;
      button.appendChild(label);

      if (item.checked) {
        const check = document.createElement('span');
        check.className = 'menu-check';
        check.innerHTML = '<svg class="icon"><use href="#icon-check"/></svg>';
        button.appendChild(check);
      }

      button.addEventListener('click', event => {
        event.stopPropagation();
        closeMenu();
        item.action?.();
      });

      menu.appendChild(button);
    }

    document.body.appendChild(menu);

    const anchorRect = anchor.getBoundingClientRect();
    const menuRect = menu.getBoundingClientRect();
    let left = alignRight ? anchorRect.right - menuRect.width : anchorRect.left;
    let top = anchorRect.bottom + 6;

    left = Math.max(8, Math.min(left, window.innerWidth - menuRect.width - 8));
    if (top + menuRect.height > window.innerHeight - 8) {
      top = Math.max(8, anchorRect.top - menuRect.height - 6);
    }

    menu.style.left = `${left}px`;
    menu.style.top = `${top}px`;

    const onPointerDown = event => {
      if (!menu.contains(event.target) && !anchor.contains(event.target)) {
        closeMenu();
      }
    };
    const onKeyDown = event => {
      if (event.key === 'Escape') {
        event.preventDefault();
        closeMenu();
        anchor.focus?.();
      }
    };
    const onResize = () => closeMenu();

    document.addEventListener('pointerdown', onPointerDown, true);
    document.addEventListener('keydown', onKeyDown, true);
    window.addEventListener('resize', onResize);
    window.addEventListener('scroll', onResize, true);

    menuCleanup = () => {
      document.removeEventListener('pointerdown', onPointerDown, true);
      document.removeEventListener('keydown', onKeyDown, true);
      window.removeEventListener('resize', onResize);
      window.removeEventListener('scroll', onResize, true);
    };

    menu.querySelector('.menu-item:not(:disabled)')?.focus({ preventScroll: true });
  }

  let modalCleanup = null;

  function openModal(options) {
    if (modalCleanup) {
      modalCleanup();
      modalCleanup = null;
    }

    return new Promise(resolve => {
      const {
        title,
        message,
        confirmText,
        cancelText,
        input,
        initial,
        danger,
        icon
      } = options;

      els.modalLayer.innerHTML = '';
      const modal = document.createElement('div');
      modal.className = 'modal';
      modal.setAttribute('role', 'dialog');
      modal.setAttribute('aria-modal', 'true');
      modal.setAttribute('aria-label', title);

      const heading = document.createElement('h2');
      heading.className = 'modal-title';
      heading.textContent = title;
      modal.appendChild(heading);

      if (message) {
        const text = document.createElement('p');
        text.className = 'modal-text';
        text.textContent = message;
        modal.appendChild(text);
      }

      let inputElement = null;
      let errorElement = null;

      if (input) {
        const field = document.createElement('div');
        field.className = 'modal-field';

        inputElement = document.createElement('input');
        inputElement.className = 'modal-input';
        inputElement.type = 'text';
        inputElement.maxLength = 60;
        inputElement.value = initial ?? '';
        inputElement.autocomplete = 'off';
        field.appendChild(inputElement);

        errorElement = document.createElement('p');
        errorElement.className = 'modal-error';
        errorElement.hidden = true;
        field.appendChild(errorElement);

        modal.appendChild(field);
      }

      const actions = document.createElement('div');
      actions.className = 'modal-actions';

      let cancelButton = null;
      if (cancelText) {
        cancelButton = document.createElement('button');
        cancelButton.type = 'button';
        cancelButton.className = 'btn btn-secondary';
        cancelButton.textContent = cancelText;
        actions.appendChild(cancelButton);
      }

      const confirmButton = document.createElement('button');
      confirmButton.type = 'button';
      confirmButton.className = `btn ${danger ? 'btn-danger' : 'btn-primary'}`;
      if (icon) {
        const iconElement = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        iconElement.setAttribute('class', 'icon');
        const useElement = document.createElementNS('http://www.w3.org/2000/svg', 'use');
        useElement.setAttribute('href', `#icon-${icon}`);
        iconElement.appendChild(useElement);
        confirmButton.appendChild(iconElement);
      }
      const confirmLabel = document.createElement('span');
      confirmLabel.textContent = confirmText;
      confirmButton.appendChild(confirmLabel);
      actions.appendChild(confirmButton);

      modal.appendChild(actions);
      els.modalLayer.appendChild(modal);
      els.modalLayer.hidden = false;

      const finish = value => {
        els.modalLayer.hidden = true;
        els.modalLayer.innerHTML = '';
        if (modalCleanup) {
          modalCleanup();
          modalCleanup = null;
        }
        resolve(value);
      };

      const confirm = () => {
        if (input) {
          const value = inputElement.value.trim();
          if (!value) {
            errorElement.textContent = t('error_name_empty');
            errorElement.hidden = false;
            inputElement.classList.add('invalid');
            inputElement.focus();
            return;
          }
          finish(value);
          return;
        }
        finish(true);
      };

      const cancel = () => finish(input ? null : false);

      confirmButton.addEventListener('click', confirm);
      cancelButton?.addEventListener('click', cancel);

      if (inputElement) {
        inputElement.addEventListener('input', () => {
          errorElement.hidden = true;
          inputElement.classList.remove('invalid');
        });
        inputElement.addEventListener('keydown', event => {
          if (event.key === 'Enter') {
            event.preventDefault();
            confirm();
          }
        });
      }

      const onKeyDown = event => {
        if (event.key === 'Escape') {
          event.preventDefault();
          event.stopPropagation();
          cancel();
        } else if (event.key === 'Tab') {
          const focusable = modal.querySelectorAll('button, input');
          if (focusable.length === 0) {
            return;
          }
          const first = focusable[0];
          const last = focusable[focusable.length - 1];
          if (event.shiftKey && document.activeElement === first) {
            event.preventDefault();
            last.focus();
          } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault();
            first.focus();
          }
        }
      };

      const onOverlayPointer = event => {
        if (event.target === els.modalLayer) {
          cancel();
        }
      };

      els.modalLayer.addEventListener('pointerdown', onOverlayPointer);
      document.addEventListener('keydown', onKeyDown, true);

      modalCleanup = () => {
        document.removeEventListener('keydown', onKeyDown, true);
        els.modalLayer.removeEventListener('pointerdown', onOverlayPointer);
      };

      window.setTimeout(() => {
        if (inputElement) {
          inputElement.focus();
          inputElement.select();
        } else {
          confirmButton.focus();
        }
      }, 30);
    });
  }

  function onListClick(event) {
    const row = event.target.closest('.account-row');
    if (!row) {
      return;
    }

    const profile = profileById(row.dataset.id);
    if (!profile) {
      return;
    }

    selectAccount(profile.id);

    if (event.target.closest('.more-button')) {
      event.stopPropagation();
      openMoreMenu(profile, event.target.closest('.more-button'));
    }
  }

  function onListDoubleClick(event) {
    if (event.target.closest('.more-button')) {
      return;
    }
    const row = event.target.closest('.account-row');
    if (!row) {
      return;
    }
    const profile = profileById(row.dataset.id);
    if (!profile) {
      return;
    }
    selectAccount(profile.id);
    activateSelected();
  }

  function onListContextMenu(event) {
    const row = event.target.closest('.account-row');
    if (!row) {
      return;
    }
    event.preventDefault();
    const profile = profileById(row.dataset.id);
    if (!profile) {
      return;
    }
    selectAccount(profile.id);
    openMoreMenu(profile, row.querySelector('.more-button') || row);
  }

  function onListKeyDown(event) {
    if (event.key === 'Enter') {
      event.preventDefault();
      activateSelected();
      return;
    }

    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      moveSelection(event.key === 'ArrowDown' ? 1 : -1);
    }
  }

  function moveSelection(direction) {
    const ids = state.profiles.map(profile => profile.id);
    if (ids.length === 0) {
      return;
    }
    const current = Math.max(0, ids.indexOf(state.selectedId));
    const next = (current + direction + ids.length) % ids.length;
    selectAccount(ids[next], { focus: true });
  }

  function onGlobalKeyDown(event) {
    if (els.modalLayer.hidden === false || document.querySelector('.menu')) {
      return;
    }

    const target = event.target;
    const inField = target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;
    const inButton = target instanceof HTMLButtonElement;

    if (event.ctrlKey && event.key.toLowerCase() === 'n') {
      event.preventDefault();
      addAccount();
      return;
    }

    if (event.key === 'F5') {
      event.preventDefault();
      refresh(false);
      return;
    }

    if (inField || inButton) {
      return;
    }

    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      moveSelection(event.key === 'ArrowDown' ? 1 : -1);
      return;
    }

    if (event.key === 'Enter') {
      event.preventDefault();
      activateSelected();
      return;
    }

    if (event.key === 'Delete') {
      event.preventDefault();
      deleteSelected(selectedProfile());
      return;
    }

    if (event.key === 'F2') {
      event.preventDefault();
      renameSelected(selectedProfile());
    }
  }

  async function init() {
    els.languageButton.addEventListener('click', openLanguageMenu);
    els.addButton.addEventListener('click', addAccount);
    els.primaryButton.addEventListener('click', activateSelected);
    els.list.addEventListener('click', onListClick);
    els.list.addEventListener('dblclick', onListDoubleClick);
    els.list.addEventListener('contextmenu', onListContextMenu);
    els.list.addEventListener('keydown', onListKeyDown);
    document.addEventListener('keydown', onGlobalKeyDown);
    window.addEventListener('focus', () => refresh(true));

    document.querySelector('.titlebar')?.addEventListener('dblclick', event => {
      if (event.target.closest('button, a, input, select')) {
        return;
      }
      toggleMaximize();
    });

    for (const handle of document.querySelectorAll('.resize-handle')) {
      handle.addEventListener('mousedown', event => {
        if (event.button !== 0 || state.busy || state.isMaximized) {
          return;
        }
        event.preventDefault();
        event.stopPropagation();
        invoke('start_resize_drag', { direction: handle.dataset.resize }).catch(() => {});
      });
    }

    els.windowMinimize.addEventListener('click', minimizeWindow);
    els.windowMaximize.addEventListener('click', toggleMaximize);
    els.windowClose.addEventListener('click', closeWindow);

    let resizeTimer = 0;
    window.addEventListener('resize', () => {
      window.clearTimeout(resizeTimer);
      resizeTimer = window.setTimeout(refreshWindowState, 120);
    });

    try {
      await listen('tray-switch', event => {
        if (state.busy) {
          return;
        }
        const id = String(event.payload ?? '');
        const profile = profileById(id);
        if (!profile) {
          return;
        }
        selectAccount(id, { focus: true });
        if (id === state.activeId) {
          openActive();
        } else {
          switchAccount(id);
        }
      });
    } catch {
    }

    await refresh(false);
    await refreshWindowState();
    window.setInterval(() => refresh(true), 2000);
  }

  init();
})();

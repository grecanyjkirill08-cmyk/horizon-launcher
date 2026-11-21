const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const state = {
    profiles: [], activeProfile: null, packs: [], filter: 'all',
    minecraftInstalled: false, minecraftVersion: '',
    pickedFolderUri: null,
};

const $ = (s) => document.querySelector(s);
const $$ = (s) => document.querySelectorAll(s);

function navigateTo(name) {
    $$('.screen').forEach((s) => s.classList.remove('active'));
    $(`#screen-${name}`)?.classList.add('active');
    $$('.nav-item').forEach((n) => n.classList.toggle('active', n.dataset.screen === name));
    if (name === 'mods') loadPacks();
    if (name === 'profiles') renderFullProfiles();
    if (name === 'settings') renderSettings();
}
$$('.nav-item').forEach((b) => b.addEventListener('click', () => navigateTo(b.dataset.screen)));

function showToast(msg, type = 'info') {
    const wrap = $('#toast-wrap');
    const t = document.createElement('div');
    t.className = `toast ${type}`;
    t.textContent = msg;
    wrap.appendChild(t);
    setTimeout(() => { t.style.opacity = '0'; t.style.transition = 'all 0.3s'; setTimeout(() => t.remove(), 300); }, 2600);
}

async function refreshMinecraftStatus() {
    try {
        const [installed, version] = await Promise.all([invoke('check_minecraft_installed'), invoke('get_minecraft_version')]);
        state.minecraftInstalled = installed;
        state.minecraftVersion = version;
        $('#mc-version').textContent = installed ? `Minecraft ${version}` : 'Не установлен';
        const st = $('#mc-status');
        st.className = `hero-status ${installed ? 'installed' : 'missing'}`;
        st.querySelector('span:last-child').textContent = installed ? 'Установлен' : 'Требуется установка';
        updatePlayButton();
    } catch (e) { showToast('Ошибка проверки Minecraft', 'error'); }
}

async function loadProfiles() {
    try {
        const [profiles, active] = await Promise.all([invoke('list_profiles'), invoke('get_active_profile')]);
        state.profiles = profiles;
        state.activeProfile = active;
        renderProfiles();
        renderFullProfiles();
        updatePlayButton();
    } catch (e) { showToast('Не удалось загрузить профили', 'error'); }
}

function renderProfiles() {
    const list = $('#profiles-list');
    if (!state.profiles.length) { list.innerHTML = `<div class="empty-state">Создайте сборку, нажав «+ Новая»</div>`; return; }
    list.innerHTML = '';
    for (const p of state.profiles) {
        const isActive = state.activeProfile?.id === p.id;
        const card = document.createElement('div');
        card.className = `profile-card${isActive ? ' active' : ''}`;
        card.innerHTML = `<div class="profile-card-badge">${isActive ? 'Активен' : 'Профиль'}</div><div class="profile-card-name">${esc(p.name)}</div><div class="profile-card-meta">${p.active_packs.length} паков</div>`;
        card.addEventListener('click', () => selectProfile(p.id));
        list.appendChild(card);
    }
}

async function selectProfile(id) {
    await invoke('set_active_profile', { id });
    await loadProfiles();
    showToast('Профиль активирован', 'success');
}

function renderFullProfiles() {
    const list = $('#profiles-full-list');
    if (!state.profiles.length) { list.innerHTML = `<div class="empty-state">Пока нет профилей</div>`; return; }
    list.innerHTML = '';
    for (const p of state.profiles) {
        const isActive = state.activeProfile?.id === p.id;
        const row = document.createElement('div');
        row.className = `profile-row${isActive ? ' active' : ''}`;
        row.innerHTML = `<div class="profile-row-info"><div class="profile-row-name">${esc(p.name)}</div><div class="profile-row-meta">${p.active_packs.length} паков • SAF URI</div></div>${isActive ? '<span class="pack-pill resources">Активен</span>' : ''}`;
        row.addEventListener('click', () => selectProfile(p.id));
        list.appendChild(row);
    }
}

async function createProfile(name, rootUri) {
    try {
        await invoke('create_profile', { name, rootUri });
        await loadProfiles();
        showToast(`Профиль «${name}» создан`, 'success');
    } catch (e) { showToast(`Ошибка: ${e}`, 'error'); }
          }
async function loadPacks() {
    if (!state.activeProfile) { $('#mods-list').innerHTML = `<div class="empty-state">Выберите активный профиль</div>`; return; }
    try {
        state.packs = await invoke('list_packs', { profileId: state.activeProfile.id });
        renderPacks();
    } catch (e) { $('#mods-list').innerHTML = `<div class="empty-state">Ошибка: ${esc(String(e))}</div>`; }
}

function renderPacks() {
    const list = $('#mods-list');
    let filtered = state.packs;
    if (state.filter === 'resources') filtered = filtered.filter((p) => p.pack_type === 'resources');
    if (state.filter === 'data') filtered = filtered.filter((p) => p.pack_type === 'data');
    if (!filtered.length) { list.innerHTML = `<div class="empty-state">Нет аддонов. Нажмите ⟰ для импорта</div>`; return; }
    list.innerHTML = '';
    for (const pack of filtered) {
        const uuid = pack.manifest.header.uuid;
        const enabled = state.activeProfile?.active_packs.includes(uuid);
        const typeIcon = pack.pack_type === 'resources' ? 'R' : 'B';
        const card = document.createElement('div');
        card.className = `mod-card${enabled ? ' enabled' : ''}`;
        card.innerHTML = `<div class="mod-icon ${pack.pack_type}">${typeIcon}</div><div class="mod-info"><div class="mod-name">${esc(pack.manifest.header.name)}</div><div class="mod-desc">${esc(pack.manifest.header.description || '')}</div><div class="mod-version">v${pack.manifest.header.version.join('.')} • ${pack.pack_type}</div></div><div class="toggle switch${enabled ? ' on' : ''}" data-uuid="${uuid}"><div class="toggle-knob"></div></div>`;
        card.querySelector('.toggle').addEventListener('click', (e) => { e.stopPropagation(); togglePack(uuid, !enabled); });
        list.appendChild(card);
    }
}

async function togglePack(uuid, active) {
    if (!state.activeProfile) return;
    try {
        await invoke('toggle_pack', { profileId: state.activeProfile.id, uuid, active });
        if (active) { if (!state.activeProfile.active_packs.includes(uuid)) state.activeProfile.active_packs.push(uuid); }
        else { state.activeProfile.active_packs = state.activeProfile.active_packs.filter((u) => u !== uuid); }
        renderPacks();
    } catch (e) { showToast(`Ошибка: ${e}`, 'error'); }
}

async function importAddon() {
    if (!state.activeProfile) { showToast('Сначала выберите профиль', 'error'); return; }
    try {
        const installed = await invoke('pick_and_install_addon', { profileId: state.activeProfile.id });
        if (!installed.length) return;
        showToast(`Установлено паков: ${installed.length}`, 'success');
        await loadPacks();
    } catch (e) { showToast(`Ошибка импорта: ${e}`, 'error'); }
}

function updatePlayButton() {
    const btn = $('#btn-play');
    $('#play-profile-name').textContent = state.activeProfile?.name ?? '—';
    if (!state.minecraftInstalled) { btn.disabled = true; btn.querySelector('.play-label').textContent = 'Нет Minecraft'; }
    else if (!state.activeProfile) { btn.disabled = true; btn.querySelector('.play-label').textContent = 'Выберите профиль'; }
    else { btn.disabled = false; btn.querySelector('.play-label').textContent = 'Играть'; }
}

async function playGame() {
    if (!state.minecraftInstalled || !state.activeProfile) return;
    try { await invoke('launch_game'); } catch (e) { showToast(`Не удалось запустить: ${e}`, 'error'); }
}

async function setupProgressListener() {
    await listen('download-progress', (event) => {
        const { percent, downloaded, total } = event.payload;
        const modal = $('#modal-progress'), fill = $('#progress-fill'), text = $('#progress-text');
        if (!modal.classList.contains('show')) modal.classList.add('show');
        const pct = Math.min(100, Math.max(0, percent || 0));
        fill.style.width = `${pct}%`;
        const mb = (downloaded / 1024 / 1024).toFixed(1);
        const totalMb = total ? (total / 1024 / 1024).toFixed(1) : '?';
        text.textContent = `${pct.toFixed(1)}% • ${mb} / ${totalMb} MB`;
        if (pct >= 100) setTimeout(() => { modal.classList.remove('show'); fill.style.width = '0%'; text.textContent = '0%'; }, 800);
    });
}

function setupModal() {
    const modal = $('#modal-new-profile');
    const open = () => { $('#input-profile-name').value = ''; state.pickedFolderUri = null; $('#display-picked-folder').textContent = 'Папка не выбрана'; modal.classList.add('show'); };
    const close = () => modal.classList.remove('show');
    $('#btn-new-profile')?.addEventListener('click', open);
    $('#btn-add-profile')?.addEventListener('click', open);
    $('#btn-modal-cancel')?.addEventListener('click', close);

    $('#btn-pick-folder')?.addEventListener('click', async () => {
        try {
            const uri = await invoke('pick_game_folder');
            if (uri) { state.pickedFolderUri = uri; $('#display-picked-folder').textContent = uri; }
        } catch (e) { showToast(`Ошибка выбора папки: ${e}`, 'error'); }
    });

    $('#btn-modal-create')?.addEventListener('click', async () => {
        const name = $('#input-profile-name').value.trim();
        if (!name) { showToast('Введите название профиля', 'error'); return; }
        if (!state.pickedFolderUri) { showToast('Выберите папку данных Minecraft', 'error'); return; }
        await createProfile(name, state.pickedFolderUri);
        close();
    });
}

function renderSettings() {
    $('#settings-mc-version').textContent = state.minecraftVersion || '—';
    $('#settings-mc-status').textContent = state.minecraftInstalled ? 'Установлен' : 'Не найден';
    $('#settings-active-profile').textContent = state.activeProfile?.name || 'Не выбран';
}

function esc(str) {
    return String(str).replace(/[&<>"']/g, (m) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[m]));
}

window.addEventListener('DOMContentLoaded', async () => {
    $$('.filter-chip').forEach((chip) => {
        chip.addEventListener('click', () => {
            $$('.filter-chip').forEach((c) => c.classList.remove('active'));
            chip.classList.add('active');
            state.filter = chip.dataset.filter;
            renderPacks();
        });
    });
    $('#btn-refresh')?.addEventListener('click', refreshMinecraftStatus);
    $('#btn-import-addon')?.addEventListener('click', importAddon);
    $('#btn-play')?.addEventListener('click', playGame);

    setupModal();
    await setupProgressListener();
    await refreshMinecraftStatus();
    await loadProfiles();
});
          

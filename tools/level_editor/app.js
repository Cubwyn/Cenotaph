"use strict";

const editorToken = loadEditorToken();

const keybindingStorageKey = "cenotaphEditorKeybindings";
const levelDraftStoragePrefix = "cenotaphLevelDraft:";
const levelDraftVersion = 1;
const movementActions = new Set([
  "cameraForward",
  "cameraBackward",
  "cameraLeft",
  "cameraRight",
  "cameraUp",
  "cameraDown",
]);
const defaultKeybindings = {
  cameraForward: { label: "Camera forward", code: "KeyW", display: "W" },
  cameraBackward: { label: "Camera backward", code: "KeyS", display: "S" },
  cameraLeft: { label: "Camera left", code: "KeyA", display: "A" },
  cameraRight: { label: "Camera right", code: "KeyD", display: "D" },
  cameraUp: { label: "Camera up", code: "KeyE", display: "E" },
  cameraDown: { label: "Camera down", code: "KeyQ", display: "Q" },
  toolSelect: { label: "Select tool", code: "Digit1", display: "1" },
  toolMove: { label: "Move tool", code: "Digit2", display: "2" },
  toolPlace: { label: "Place tool", code: "Digit3", display: "3" },
  toolDraw: { label: "Draw tool", code: "Digit4", display: "4" },
  undo: { label: "Undo", code: "KeyZ", ctrl: true, display: "Ctrl+Z" },
  redo: { label: "Redo", code: "KeyY", ctrl: true, display: "Ctrl+Y" },
  redoAlt: { label: "Redo alternate", code: "KeyZ", ctrl: true, shift: true, display: "Ctrl+Shift+Z" },
  focusSelected: { label: "Focus selected", code: "KeyF", display: "F" },
  duplicateSelected: { label: "Duplicate selected", code: "KeyD", ctrl: true, display: "Ctrl+D" },
  copySelected: { label: "Copy selected", code: "KeyC", ctrl: true, display: "Ctrl+C" },
  pasteSelected: { label: "Paste selected", code: "KeyV", ctrl: true, display: "Ctrl+V" },
  selectAll: { label: "Select all props", code: "KeyA", ctrl: true, display: "Ctrl+A" },
  validate: { label: "Validate", code: "KeyV", display: "V" },
  save: { label: "Save", code: "KeyS", ctrl: true, display: "Ctrl+S" },
  deleteSelected: { label: "Delete selected", code: "Delete", display: "Delete" },
  resetCamera: { label: "Reset camera", code: "Home", display: "Home" },
  toggleLayout: { label: "Toggle 4 View", code: "KeyT", display: "T" },
  nudgeLeft: { label: "Nudge -X", code: "ArrowLeft", display: "Left" },
  nudgeRight: { label: "Nudge +X", code: "ArrowRight", display: "Right" },
  nudgeForward: { label: "Nudge -Z", code: "ArrowUp", display: "Up" },
  nudgeBackward: { label: "Nudge +Z", code: "ArrowDown", display: "Down" },
  nudgeUp: { label: "Nudge +Y", code: "PageUp", display: "PageUp" },
  nudgeDown: { label: "Nudge -Y", code: "PageDown", display: "PageDown" },
  cancel: { label: "Cancel/menu close", code: "Escape", display: "Esc" },
};
const brushFaces = [
  [0, 1, 3], [0, 3, 2],
  [4, 6, 7], [4, 7, 5],
  [0, 4, 5], [0, 5, 1],
  [2, 3, 7], [2, 7, 6],
  [0, 2, 6], [0, 6, 4],
  [1, 5, 7], [1, 7, 3],
];
const brushEdges = [
  [0, 1], [0, 2], [1, 3], [2, 3],
  [4, 5], [4, 6], [5, 7], [6, 7],
  [0, 4], [1, 5], [2, 6], [3, 7],
];
const brushVertexLabels = [
  "-X -Y -Z", "-X -Y +Z", "-X +Y -Z", "-X +Y +Z",
  "+X -Y -Z", "+X -Y +Z", "+X +Y -Z", "+X +Y +Z",
];

const state = {
  project: null,
  levelId: null,
  level: null,
  selectedProp: null,
  selectedProps: new Set(),
  selectionAnchor: null,
  selectedTemplate: null,
  prefab: {
    selectedId: null,
    data: null,
    loading: false,
  },
  lastPlacementLabel: null,
  tool: "select",
  dirty: false,
  projectLoading: false,
  levelLoading: false,
  levelLoadSerial: 0,
  connection: {
    ready: false,
    message: "Connecting to the editor server...",
  },
  validation: { current: false, ok: false, errors: [] },
  camera: {
    position: [0, 140, 28],
    yaw: 0,
    pitch: -0.35,
    speed: 18,
  },
  viewLayout: "quad",
  ortho: {
    center: [0, 126, 0],
    zoom: 4,
  },
  keys: new Set(),
  keybindings: loadKeybindings(),
  capturingBinding: null,
  clipboardProps: [],
  clipboardCenter: null,
  history: {
    past: [],
    future: [],
    current: null,
    saved: null,
    applying: false,
    limit: 80,
    transactionDepth: 0,
    transactionChanged: false,
  },
  draft: {
    timer: null,
    pending: false,
    savedAt: null,
    recovered: false,
    available: true,
    warned: false,
  },
  gridSize: 1,
  workspaceTab: "create",
  transform: {
    axis: "all",
    snap: true,
  },
  renderer: null,
  drag: null,
  suppressContextMenu: false,
  suppressContextMenuUntil: 0,
  contextMenu: {
    world: null,
    propIndex: null,
    viewName: "camera",
  },
  drawBrush: {
    kind: "floor",
    viewName: "top",
    height: 3,
    thickness: 0.5,
    direction: "x+",
    segments: 12,
    steps: 6,
    groundY: 126,
    frontZ: 0,
    sideX: 0,
    terrainResolution: 8,
    terrainRelief: 3,
    terrainSeed: 1,
    terrainStrength: 1,
    start: null,
    current: null,
  },
  needsDraw: true,
  lastFrameMs: performance.now(),
};

const el = {
  projectSummary: document.querySelector("#projectSummary"),
  connectionBanner: document.querySelector("#connectionBanner"),
  connectionMessage: document.querySelector("#connectionMessage"),
  reconnectEditor: document.querySelector("#reconnectEditor"),
  refreshProject: document.querySelector("#refreshProject"),
  newLevel: document.querySelector("#newLevel"),
  duplicateLevel: document.querySelector("#duplicateLevel"),
  undoAction: document.querySelector("#undoAction"),
  redoAction: document.querySelector("#redoAction"),
  validateLevel: document.querySelector("#validateLevel"),
  saveLevel: document.querySelector("#saveLevel"),
  levelStatus: document.querySelector("#levelStatus"),
  levelList: document.querySelector("#levelList"),
  objectCount: document.querySelector("#objectCount"),
  objectFilter: document.querySelector("#objectFilter"),
  objectList: document.querySelector("#objectList"),
  viewport: document.querySelector("#viewport"),
  viewportGrid: document.querySelector("#viewportGrid"),
  topView: document.querySelector("#topView"),
  frontView: document.querySelector("#frontView"),
  sideView: document.querySelector("#sideView"),
  topViewLabel: document.querySelector("#topViewLabel"),
  frontViewLabel: document.querySelector("#frontViewLabel"),
  sideViewLabel: document.querySelector("#sideViewLabel"),
  viewportHint: document.querySelector("#viewportHint"),
  toolSelect: document.querySelector("#toolSelect"),
  toolMove: document.querySelector("#toolMove"),
  toolPlace: document.querySelector("#toolPlace"),
  toolDraw: document.querySelector("#toolDraw"),
  layoutQuad: document.querySelector("#layoutQuad"),
  layoutCamera: document.querySelector("#layoutCamera"),
  resetCamera: document.querySelector("#resetCamera"),
  transformAxis: document.querySelector("#transformAxis"),
  snapToggle: document.querySelector("#snapToggle"),
  selectionSummary: document.querySelector("#selectionSummary"),
  brushKind: document.querySelector("#brushKind"),
  brushHeight: document.querySelector("#brushHeight"),
  brushThickness: document.querySelector("#brushThickness"),
  brushDirection: document.querySelector("#brushDirection"),
  brushSegments: document.querySelector("#brushSegments"),
  brushSteps: document.querySelector("#brushSteps"),
  terrainResolution: document.querySelector("#terrainResolution"),
  terrainRelief: document.querySelector("#terrainRelief"),
  terrainSeed: document.querySelector("#terrainSeed"),
  brushGround: document.querySelector("#brushGround"),
  brushFrontZ: document.querySelector("#brushFrontZ"),
  brushSideX: document.querySelector("#brushSideX"),
  gridSize: document.querySelector("#gridSize"),
  paletteStatus: document.querySelector("#paletteStatus"),
  templateList: document.querySelector("#templateList"),
  assetCount: document.querySelector("#assetCount"),
  assetKindFilter: document.querySelector("#assetKindFilter"),
  assetFilter: document.querySelector("#assetFilter"),
  assetList: document.querySelector("#assetList"),
  prefabCount: document.querySelector("#prefabCount"),
  prefabFilter: document.querySelector("#prefabFilter"),
  prefabList: document.querySelector("#prefabList"),
  prefabStatus: document.querySelector("#prefabStatus"),
  prefabName: document.querySelector("#prefabName"),
  prefabId: document.querySelector("#prefabId"),
  createPrefab: document.querySelector("#createPrefab"),
  deletePrefab: document.querySelector("#deletePrefab"),
  resetKeybindings: document.querySelector("#resetKeybindings"),
  keybindingList: document.querySelector("#keybindingList"),
  inspector: document.querySelector("#inspector"),
  assetImportsInspector: document.querySelector("#assetImportsInspector"),
  eventsInspector: document.querySelector("#eventsInspector"),
  lootInspector: document.querySelector("#lootInspector"),
  pathsInspector: document.querySelector("#pathsInspector"),
  dialogueInspector: document.querySelector("#dialogueInspector"),
  workspaceTabs: [...document.querySelectorAll("[data-editor-tab]")],
  workspacePanels: [...document.querySelectorAll("[data-editor-panel]")],
  brushFields: [...document.querySelectorAll("[data-brush-field]")],
  deleteSelected: document.querySelector("#deleteSelected"),
  validationBadge: document.querySelector("#validationBadge"),
  validationList: document.querySelector("#validationList"),
  contextMenu: document.querySelector("#contextMenu"),
};

const baseTemplates = [
  {
    group: "geometry",
    label: "Wall",
    asset_id: "props/test_wall.obj",
    scale: [8, 3, 1],
    collider_type: "Box",
  },
  {
    group: "geometry",
    label: "Floor",
    asset_id: "props/test_platform.obj",
    scale: [8, 0.5, 8],
    collider_type: "Box",
  },
  {
    group: "geometry",
    label: "Pillar",
    asset_id: "props/test_obelisk.obj",
    scale: [1.2, 3, 1.2],
    collider_type: "Box",
  },
  {
    group: "item",
    label: "Resource shard",
    asset_id: "pickups/resource_shard.obj",
    scale: [0.35, 0.35, 0.35],
    resource_value: 25,
  },
  {
    group: "entity",
    label: "Anchor",
    asset_id: "world/anchor_marker.obj",
    scale: [0.8, 2.5, 0.8],
    anchor_id: "anchor",
  },
  {
    group: "entity",
    label: "Hazard",
    asset_id: "world/hurtbox_warning.obj",
    scale: [1.5, 1.5, 1.5],
    collider_type: "Sphere",
    is_hurtbox: true,
  },
  {
    group: "entity",
    label: "Transition gate",
    asset_id: "world/transition_gate.obj",
    scale: [1, 2, 1],
    trigger_level_id: "movement_test",
  },
];

function levelDefaults(id = "new_level") {
  return {
    name: titleCase(id.replaceAll("_", " ")),
    base_map: "assets/test_movement_arena.obj",
    player_spawn: [0, 128, 0],
    props: [],
    asset_imports: [],
    loot_tables: [],
    paths: [],
    events: [],
    dialogues: [],
  };
}

function propDefaults(position = [0, 126, 0]) {
  return {
    asset_id: "props/test_wall.obj",
    position,
    rotation: [0, 0, 0],
    scale: [1, 1, 1],
    collider_type: "None",
    is_climbable: false,
    is_hurtbox: false,
    item_id: null,
    resource_value: 0,
    anchor_id: null,
    enemy_type: null,
    enemy_health: 0,
    light_color: null,
    light_intensity: 0,
    ambient_sound_id: null,
    trigger_level_id: null,
    loot_table_id: null,
    path_id: null,
    dialogue_id: null,
    event_id: null,
  };
}

async function api(path, options = {}) {
  if (!editorToken) {
    const error = new Error("Editor session token is missing. Reopen the tokenized URL printed by cargo run -- editor.");
    error.status = 403;
    error.connectionFailure = true;
    setConnectionError(error.message);
    throw error;
  }
  const { headers, ...requestOptions } = options;
  let response;
  try {
    response = await fetch(path, {
      ...requestOptions,
      headers: {
        "Content-Type": "application/json",
        "X-Cenotaph-Editor-Token": editorToken,
        ...(headers || {}),
      },
    });
  } catch (cause) {
    const error = new Error("Cannot reach the editor server. Check that cargo run -- editor is still running.");
    error.connectionFailure = true;
    error.cause = cause;
    setConnectionError(error.message);
    throw error;
  }

  let payload;
  try {
    payload = await response.json();
  } catch (cause) {
    const error = new Error(`Editor server returned an invalid response (${response.status}).`);
    error.status = response.status;
    error.connectionFailure = true;
    error.cause = cause;
    setConnectionError(error.message);
    throw error;
  }
  if (!payload || typeof payload !== "object" || Array.isArray(payload)) {
    const error = new Error(`Editor server returned an unexpected payload (${response.status}).`);
    error.status = response.status;
    error.connectionFailure = true;
    setConnectionError(error.message);
    throw error;
  }
  if (!response.ok || payload.ok === false) {
    const message = payload.error || (payload.errors || ["request failed"]).join("; ");
    const error = new Error(message);
    error.status = response.status;
    if (response.status === 401 || response.status === 403) {
      error.connectionFailure = true;
      setConnectionError("Editor session expired or is invalid. Reconnect with the tokenized URL printed by cargo run -- editor.");
    }
    throw error;
  }
  setConnectionOk();
  return payload;
}

function loadEditorToken() {
  const query = new URLSearchParams(window.location.search);
  const fragment = new URLSearchParams(window.location.hash.slice(1));
  const queryToken = query.get("token");
  const token = queryToken || fragment.get("token") || sessionStorage.getItem("cenotaphEditorToken");
  if (token) {
    sessionStorage.setItem("cenotaphEditorToken", token);
  }
  if (queryToken) {
    const url = new URL(window.location.href);
    url.searchParams.delete("token");
    const nextFragment = new URLSearchParams(url.hash.slice(1));
    nextFragment.set("token", queryToken);
    url.hash = nextFragment.toString();
    history.replaceState(null, "", url);
  }
  return token;
}

function setConnectionOk() {
  state.connection = { ready: true, message: "Connected" };
  if (el.connectionBanner) {
    el.connectionBanner.hidden = true;
  }
  renderAvailabilityControls();
}

function setConnectionError(message) {
  state.connection = { ready: false, message };
  if (el.connectionBanner) {
    el.connectionBanner.hidden = false;
    el.connectionMessage.textContent = message;
  }
  if (!state.level) {
    state.tool = "select";
    state.drawBrush.start = null;
    state.drawBrush.current = null;
    if (state.drag?.kind === "draw") {
      state.drag = null;
    }
    updateViewportHint("Reconnect the editor session before loading or editing levels.");
  }
  renderAvailabilityControls();
  draw();
}

function reconnectEditor() {
  if (state.dirty && !confirm("Reconnect and discard unsaved editor changes?")) {
    return;
  }
  const input = prompt("Paste the full tokenized editor URL printed in the terminal, or paste its token.", "");
  if (!input) {
    return;
  }
  const token = editorTokenFromInput(input);
  if (!token) {
    setConnectionError("That URL or token is not valid. Use the current URL printed by cargo run -- editor.");
    return;
  }
  if (state.dirty) {
    clearLocalDraft(state.levelId);
  }
  sessionStorage.setItem("cenotaphEditorToken", token);
  const url = new URL(window.location.href);
  url.searchParams.delete("token");
  const fragment = new URLSearchParams(url.hash.slice(1));
  fragment.set("token", token);
  url.hash = fragment.toString();
  history.replaceState(null, "", url);
  window.location.reload();
}

function editorTokenFromInput(value) {
  const trimmed = String(value || "").trim();
  if (!trimmed) {
    return null;
  }
  try {
    const parsed = new URL(trimmed);
    const fragment = new URLSearchParams(parsed.hash.slice(1));
    const token = parsed.searchParams.get("token") || fragment.get("token");
    if (token && /^[a-f0-9]{32,}$/i.test(token)) {
      return token;
    }
  } catch (_) {
    // Raw tokens are accepted below.
  }
  return /^[a-f0-9]{32,}$/i.test(trimmed) ? trimmed : null;
}

function loadKeybindings() {
  let saved = {};
  try {
    saved = JSON.parse(localStorage.getItem(keybindingStorageKey) || "{}");
  } catch (_) {
    saved = {};
  }
  const bindings = {};
  for (const [action, binding] of Object.entries(defaultKeybindings)) {
    bindings[action] = normalizeBinding({ ...binding, ...(saved[action] || {}) }, binding);
  }
  return bindings;
}

function saveKeybindings() {
  localStorage.setItem(keybindingStorageKey, JSON.stringify(state.keybindings));
}

function resetKeybindings() {
  localStorage.removeItem(keybindingStorageKey);
  state.keybindings = loadKeybindings();
  state.capturingBinding = null;
  renderKeybindings();
  updateViewportHint("Keybindings reset.");
}

function levelDraftStorageKey(id) {
  return `${levelDraftStoragePrefix}${id}`;
}

function cancelLocalDraftTimer() {
  if (state.draft.timer != null) {
    window.clearTimeout(state.draft.timer);
    state.draft.timer = null;
  }
}

function resetLocalDraftStatus() {
  cancelLocalDraftTimer();
  state.draft.pending = false;
  state.draft.savedAt = null;
  state.draft.recovered = false;
}

function removeLocalDraftStorage(id) {
  if (!id) {
    return;
  }
  try {
    localStorage.removeItem(levelDraftStorageKey(id));
  } catch (_) {
    // Storage can be unavailable in hardened browser profiles.
  }
}

function clearLocalDraft(id) {
  removeLocalDraftStorage(id);
  if (id === state.levelId) {
    resetLocalDraftStatus();
    renderLevelStatus();
  }
}

function queueLocalDraft() {
  if (!state.levelId || !state.level) {
    return;
  }
  if (!state.dirty) {
    clearLocalDraft(state.levelId);
    return;
  }
  if (!state.draft.available) {
    return;
  }
  cancelLocalDraftTimer();
  state.draft.pending = true;
  state.draft.timer = window.setTimeout(persistLocalDraft, 450);
  renderLevelStatus();
}

function persistLocalDraft() {
  cancelLocalDraftTimer();
  if (!state.levelId || !state.level || !state.dirty || !state.draft.available) {
    state.draft.pending = false;
    return;
  }
  const snapshot = snapshotLevel();
  if (!snapshot) {
    state.draft.pending = false;
    return;
  }
  const savedAt = new Date().toISOString();
  const payload = {
    version: levelDraftVersion,
    levelId: state.levelId,
    savedAt,
    baseSnapshot: state.history.saved,
    level: JSON.parse(snapshot),
  };
  try {
    localStorage.setItem(levelDraftStorageKey(state.levelId), JSON.stringify(payload));
    state.draft.pending = false;
    state.draft.savedAt = savedAt;
  } catch (_) {
    state.draft.pending = false;
    state.draft.available = false;
    if (!state.draft.warned) {
      state.draft.warned = true;
      updateViewportHint("Local draft storage is unavailable. Save the level before reloading the editor.");
    }
  }
  renderLevelStatus();
}

function readLocalDraft(id, baseSnapshot) {
  let payload;
  try {
    const raw = localStorage.getItem(levelDraftStorageKey(id));
    if (!raw) {
      return null;
    }
    payload = JSON.parse(raw);
    if (
      !payload
      || payload.version !== levelDraftVersion
      || payload.levelId !== id
      || !payload.level
      || typeof payload.level !== "object"
      || Array.isArray(payload.level)
      || (payload.baseSnapshot !== null && typeof payload.baseSnapshot !== "string")
    ) {
      throw new Error("Malformed local draft");
    }
    const level = normalizeLevel(payload.level);
    const snapshot = JSON.stringify(level);
    if (snapshot === baseSnapshot) {
      removeLocalDraftStorage(id);
      return null;
    }
    const savedAt = typeof payload.savedAt === "string" && Number.isFinite(Date.parse(payload.savedAt))
      ? payload.savedAt
      : null;
    return {
      level,
      snapshot,
      savedAt,
      diskChanged: payload.baseSnapshot !== baseSnapshot,
    };
  } catch (_) {
    removeLocalDraftStorage(id);
    return null;
  }
}

function normalizeBinding(binding, fallback) {
  const code = typeof binding.code === "string" && binding.code ? binding.code : fallback.code;
  const normalized = {
    label: fallback.label,
    code,
    ctrl: !!binding.ctrl,
    alt: !!binding.alt,
    shift: !!binding.shift,
  };
  normalized.display = keyDisplay(normalized);
  return normalized;
}

function keyDisplay(binding) {
  const parts = [];
  if (binding.ctrl) parts.push("Ctrl");
  if (binding.alt) parts.push("Alt");
  if (binding.shift) parts.push("Shift");
  parts.push(codeDisplay(binding.code));
  return parts.join("+");
}

function codeDisplay(code) {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (code === "Escape") return "Esc";
  if (code === "Space") return "Space";
  if (code.startsWith("Arrow")) return code.replace("Arrow", "");
  return code;
}

function eventBinding(event) {
  return {
    code: event.code,
    ctrl: event.ctrlKey,
    alt: event.altKey,
    shift: event.shiftKey,
  };
}

function captureBinding(event) {
  if (!state.capturingBinding) {
    return false;
  }
  event.preventDefault();
  if (event.code === "Escape") {
    state.capturingBinding = null;
    renderKeybindings();
    updateViewportHint();
    return true;
  }
  state.keybindings[state.capturingBinding] = normalizeBinding(
    eventBinding(event),
    defaultKeybindings[state.capturingBinding]
  );
  saveKeybindings();
  const label = defaultKeybindings[state.capturingBinding].label;
  state.capturingBinding = null;
  renderKeybindings();
  updateViewportHint(`${label} rebound.`);
  return true;
}

function actionForEvent(event) {
  for (const [action, binding] of Object.entries(state.keybindings)) {
    if (bindingMatchesEvent(binding, event)) {
      return action;
    }
  }
  return null;
}

function bindingMatchesEvent(binding, event) {
  if (binding.code !== event.code) {
    return false;
  }
  if (!!binding.ctrl !== event.ctrlKey) {
    return false;
  }
  if (!!binding.alt !== event.altKey) {
    return false;
  }
  if (!!binding.shift !== event.shiftKey) {
    return false;
  }
  return true;
}

async function refreshProject() {
  if (state.projectLoading) {
    return false;
  }
  state.projectLoading = true;
  renderAvailabilityControls();
  setProjectStatus("Loading project...");
  try {
    state.project = await api("/api/project");
    if (!Array.isArray(state.project.levels)) {
      throw new Error("Project response is missing its level list.");
    }
    state.project.prefabs = Array.isArray(state.project.prefabs) ? state.project.prefabs : [];
    const prefabIds = new Set(state.project.prefabs.map((prefab) => prefab.id));
    if (state.prefab.selectedId && !prefabIds.has(state.prefab.selectedId)) {
      clearSelectedPrefab();
    }
    renderProject();
    renderPalette();
    renderAssetBrowser();
    renderPrefabs();
    if (!state.level && state.project.levels.length > 0) {
      await loadLevel(state.project.levels[0].id);
    }
    if (!state.level && state.project.levels.length === 0) {
      updateViewportHint("Create a level to begin editing.");
    }
    return true;
  } catch (error) {
    setProjectStatus(`Project load failed: ${error.message}`);
    updateViewportHint("Project loading failed. Reconnect the session, then refresh the project.");
    return false;
  } finally {
    state.projectLoading = false;
    renderAvailabilityControls();
    renderProjectLevelButtons();
  }
}

async function loadLevel(id) {
  if (!id || state.levelLoading) {
    return false;
  }
  const discardedLevelId = state.dirty ? state.levelId : null;
  if (state.dirty && !confirm("Discard unsaved editor changes?")) {
    return false;
  }
  const requestSerial = ++state.levelLoadSerial;
  state.levelLoading = true;
  renderProject();
  renderAvailabilityControls();
  el.levelStatus.textContent = `Loading ${id}...`;
  setProjectStatus(`Loading level ${id}...`);
  try {
    const payload = await api(`/api/levels/${encodeURIComponent(id)}`);
    if (requestSerial !== state.levelLoadSerial) {
      return false;
    }
    if (!payload.id || !payload.level || typeof payload.level !== "object" || Array.isArray(payload.level)) {
      throw new Error(`Level ${id} returned malformed editor data.`);
    }
    const serverLevel = normalizeLevel(payload.level);
    const baseSnapshot = JSON.stringify(serverLevel);
    if (discardedLevelId) {
      clearLocalDraft(discardedLevelId);
    }
    resetLocalDraftStatus();
    const draft = readLocalDraft(payload.id, baseSnapshot);
    let recoveredDraft = false;
    if (draft) {
      const savedLabel = draft.savedAt ? new Date(draft.savedAt).toLocaleString() : "an earlier session";
      const conflictWarning = draft.diskChanged
        ? "\n\nThe saved level changed after this draft began. Recovering keeps both versions available through Undo."
        : "";
      recoveredDraft = confirm(`Recover the unsaved local draft from ${savedLabel}?${conflictWarning}`);
      if (!recoveredDraft) {
        removeLocalDraftStorage(payload.id);
      }
    }
    state.levelId = payload.id;
    state.level = recoveredDraft ? draft.level : serverLevel;
    resetSelection();
    state.dirty = recoveredDraft;
    state.draft.savedAt = recoveredDraft ? draft.savedAt : null;
    state.draft.recovered = recoveredDraft;
    state.validation = { current: false, ok: false, errors: [] };
    resetBrushWorkPlanes();
    resetCameraToLevel();
    resetHistory(recoveredDraft ? baseSnapshot : undefined);
    if (recoveredDraft) {
      state.history.past = [baseSnapshot];
    }
    state.levelLoading = false;
    renderAll();
    await validateLevel(false);
    updateViewportHint(recoveredDraft
      ? `Recovered the local draft for ${state.level.name || state.levelId}. Undo returns to the saved level.`
      : `Loaded ${state.level.name || state.levelId}.`);
    return true;
  } catch (error) {
    if (requestSerial === state.levelLoadSerial) {
      setProjectStatus(`Level load failed: ${error.message}`);
      el.levelStatus.textContent = state.levelId
        ? `${state.levelId}${state.dirty ? " · Unsaved" : " · Saved"}`
        : "No level";
      updateViewportHint(`Could not load ${id}: ${error.message}`);
    }
    return false;
  } finally {
    if (requestSerial === state.levelLoadSerial) {
      state.levelLoading = false;
      renderAvailabilityControls();
      renderProjectLevelButtons();
    }
  }
}

async function saveLevel() {
  if (!state.level) {
    return;
  }
  const id = state.levelId || prompt("Level id to save as", "new_level");
  if (!id) {
    return;
  }
  try {
    const previousId = state.levelId;
    await api(`/api/levels/${encodeURIComponent(id)}`, {
      method: "PUT",
      body: JSON.stringify(state.level),
    });
    if (previousId && previousId !== id) {
      removeLocalDraftStorage(previousId);
    }
    state.levelId = id;
    state.dirty = false;
    clearLocalDraft(id);
    resetHistory();
    state.validation = { current: true, ok: true, errors: [] };
    await refreshProject();
    renderAll();
    setValidationOk("Saved and validated.");
  } catch (error) {
    const errors = error.message.split("; ").filter(Boolean);
    state.validation = { current: true, ok: false, errors };
    setWorkspaceTab("validation");
    renderValidation();
  }
}

async function validateLevel(showOk = true) {
  if (!state.level) {
    return false;
  }
  try {
    const payload = await api("/api/validate", {
      method: "POST",
      body: JSON.stringify(state.level),
    });
    state.validation = {
      current: true,
      ok: payload.ok,
      errors: payload.errors || [],
    };
    renderValidation();
    if (payload.ok && showOk) {
      setValidationOk("Level validation passed.");
    }
    return payload.ok;
  } catch (error) {
    state.validation = { current: true, ok: false, errors: [error.message] };
    renderValidation();
    return false;
  }
}

function newLevel() {
  if (!state.connection.ready) {
    updateViewportHint("Reconnect the editor session before creating a level.");
    return;
  }
  if (state.dirty && !confirm("Discard unsaved editor changes?")) {
    return;
  }
  const id = sanitizeLevelId(prompt("New level id", "new_level"));
  if (!id) {
    updateViewportHint("New level cancelled: level id is required.");
    return;
  }
  if (levelIdExists(id) && !confirm(`Level '${id}' already exists. Continue and overwrite it when saved?`)) {
    return;
  }
  if (state.dirty) {
    clearLocalDraft(state.levelId);
  }
  state.levelId = id;
  removeLocalDraftStorage(id);
  resetLocalDraftStatus();
  state.level = levelDefaults(id);
  resetSelection();
  state.dirty = true;
  resetBrushWorkPlanes();
  resetCameraToLevel();
  resetHistory();
  queueLocalDraft();
  markValidationStale();
  renderAll();
}

function duplicateLevel() {
  if (!state.connection.ready) {
    updateViewportHint("Reconnect the editor session before duplicating a level.");
    return;
  }
  if (!state.level) {
    updateViewportHint("Load a level before duplicating it.");
    return;
  }
  const id = sanitizeLevelId(prompt("Duplicate level as", `${state.levelId || "level"}_copy`));
  if (!id) {
    updateViewportHint("Duplicate cancelled: level id is required.");
    return;
  }
  if (levelIdExists(id) && !confirm(`Level '${id}' already exists. Continue and overwrite it when saved?`)) {
    return;
  }
  if (state.dirty) {
    persistLocalDraft();
  }
  state.levelId = id;
  removeLocalDraftStorage(id);
  resetLocalDraftStatus();
  state.level = normalizeLevel(JSON.parse(JSON.stringify({
    ...state.level,
    name: `${state.level.name || state.levelId} Copy`,
  })));
  resetSelection();
  state.dirty = true;
  resetCameraToLevel();
  resetHistory();
  queueLocalDraft();
  markValidationStale();
  renderAll();
  updateViewportHint(`Duplicated level as ${id}. Save when ready.`);
}

function normalizeLevel(level) {
  const normalized = { ...level };
  normalized.player_spawn = normalized.player_spawn || [0, 128, 0];
  normalized.props = (normalized.props || []).map((prop) => {
    const normalizedProp = { ...propDefaults(), ...prop };
    if (normalizedProp.brush_geometry) {
      normalizeBrushGeometry(normalizedProp.brush_geometry);
    }
    return normalizedProp;
  });
  for (const key of ["asset_imports", "loot_tables", "paths", "events", "dialogues"]) {
    normalized[key] = normalized[key] || [];
  }
  return normalized;
}

function renderAll() {
  renderProject();
  renderObjects();
  renderAssetBrowser();
  renderPrefabs();
  renderKeybindings();
  renderHistoryControls();
  renderWorkspaceTabs();
  renderBrushControls();
  renderSelectionSummary();
  renderAvailabilityControls();
  renderInspector();
  renderSystemPanels();
  renderValidation();
  draw();
}

function renderProject() {
  if (!state.project) {
    renderLevelStatus();
    return;
  }
  const catalog = projectAssetCatalog();
  const totalAssets = catalog.assets.length || state.project.assets.length;
  setProjectStatus(
    `${state.project.levels.length} levels, ${(state.project.prefabs || []).length} prefabs, ${totalAssets} files, ${catalog.models.length || state.project.assets.length} models, ${catalog.textures.length} textures, ${state.project.enemies.length} enemies, ${state.project.relics.length} relics`
  );
  renderLevelStatus();
  renderProjectLevelButtons();
}

function renderLevelStatus() {
  if (!el.levelStatus) {
    return;
  }
  if (!state.levelId) {
    el.levelStatus.textContent = "No level";
    el.levelStatus.removeAttribute("title");
    return;
  }
  let status = state.dirty ? " · Unsaved" : " · Saved";
  if (state.dirty && state.draft.pending) {
    status += " · Draft pending";
  } else if (state.dirty && state.draft.savedAt) {
    status += " · Draft saved locally";
  } else if (state.dirty && state.draft.recovered) {
    status += " · Draft recovered";
  }
  el.levelStatus.textContent = `${state.levelId}${status}`;
  if (state.draft.savedAt) {
    el.levelStatus.title = `Local draft saved ${new Date(state.draft.savedAt).toLocaleString()}`;
  } else {
    el.levelStatus.removeAttribute("title");
  }
}

function renderProjectLevelButtons() {
  if (!state.project) {
    return;
  }
  el.levelList.innerHTML = "";
  for (const level of state.project.levels) {
    const button = document.createElement("button");
    button.className = `level-card ${level.id === state.levelId ? "active" : ""}`;
    button.innerHTML = `<span class="card-title">${escapeHtml(level.name || level.id)}</span><span class="card-meta">${escapeHtml(level.id)} · ${level.props} props</span>`;
    button.disabled = state.levelLoading || state.projectLoading;
    button.addEventListener("click", () => loadLevel(level.id));
    el.levelList.append(button);
  }
}

function levelIdExists(id) {
  return (state.project?.levels || []).some((level) => level.id === id);
}

function renderWorkspaceTabs() {
  for (const button of el.workspaceTabs) {
    const active = button.dataset.editorTab === state.workspaceTab;
    button.classList.toggle("active", active);
    button.setAttribute("aria-selected", String(active));
  }
  for (const panel of el.workspacePanels) {
    panel.hidden = panel.dataset.editorPanel !== state.workspaceTab;
  }
}

function setWorkspaceTab(tab) {
  if (!el.workspacePanels.some((panel) => panel.dataset.editorPanel === tab)) {
    return;
  }
  state.workspaceTab = tab;
  renderWorkspaceTabs();
  draw();
  requestAnimationFrame(draw);
}

function renderBrushControls() {
  const kind = el.brushKind.value;
  for (const field of el.brushFields) {
    field.hidden = !field.dataset.brushField.split(/\s+/).includes(kind);
  }
  el.snapToggle.classList.toggle("active", state.transform.snap);
  el.snapToggle.setAttribute("aria-pressed", String(state.transform.snap));
  el.transformAxis.value = state.transform.axis;
  el.topViewLabel.textContent = `Top X/Z · Y ${formatNumber(state.drawBrush.groundY)}`;
  el.frontViewLabel.textContent = `Front X/Y · Z ${formatNumber(state.drawBrush.frontZ)}`;
  el.sideViewLabel.textContent = `Side Z/Y · X ${formatNumber(state.drawBrush.sideX)}`;
}

function renderSystemPanels() {
  const panels = [
    [el.assetImportsInspector, "Asset Imports", "asset_imports", addAssetImportStub, assetImportCard],
    [el.eventsInspector, "Events", "events", addEvent, eventCard],
    [el.lootInspector, "Loot Tables", "loot_tables", addLootTable, lootTableCard],
    [el.pathsInspector, "Paths", "paths", addPath, pathCard],
    [el.dialogueInspector, "Dialogues", "dialogues", addDialogue, dialogueCard],
  ];
  for (const [panel, title, key, addFn, cardFn] of panels) {
    if (!panel) {
      continue;
    }
    if (!state.level) {
      panel.className = "inspector empty";
      panel.textContent = "Load or create a level.";
      continue;
    }
    panel.className = "";
    panel.replaceChildren(systemSection(title, key, addFn, cardFn));
  }
}

function selectionIndices() {
  return [...state.selectedProps]
    .filter((index) => Number.isInteger(index) && !!state.level?.props?.[index])
    .sort((left, right) => left - right);
}

function selectionContains(index) {
  return state.selectedProps.has(index) && !!state.level?.props?.[index];
}

function resetSelection() {
  state.selectedProp = null;
  state.selectedProps.clear();
  state.selectionAnchor = null;
}

function setSelection(indices, primary = null) {
  const valid = [...new Set(indices)]
    .filter((index) => Number.isInteger(index) && !!state.level?.props?.[index])
    .sort((left, right) => left - right);
  state.selectedProps = new Set(valid);
  state.selectedProp = valid.includes(primary) ? primary : (valid.at(-1) ?? null);
  if (state.selectedProp != null) {
    state.selectionAnchor = state.selectedProp;
  } else {
    state.selectionAnchor = null;
  }
  renderSelectionSummary();
}

function selectProp(index, options = {}) {
  if (index == null || !state.level?.props?.[index]) {
    if (!options.additive) {
      resetSelection();
    }
    return;
  }
  const current = new Set(selectionIndices());
  if (options.range && state.selectionAnchor != null) {
    const start = Math.min(state.selectionAnchor, index);
    const end = Math.max(state.selectionAnchor, index);
    const range = Array.from({ length: end - start + 1 }, (_, offset) => start + offset);
    setSelection(options.additive ? [...current, ...range] : range, index);
    return;
  }
  if (options.toggle) {
    if (current.has(index)) {
      current.delete(index);
      setSelection([...current], [...current].at(-1) ?? null);
    } else {
      current.add(index);
      setSelection([...current], index);
    }
    return;
  }
  setSelection(options.additive ? [...current, index] : [index], index);
}

function selectAllProps() {
  if (!state.level) {
    return;
  }
  setSelection(state.level.props.map((_, index) => index), state.level.props.length - 1);
  renderAll();
  updateViewportHint(`${selectionIndices().length} props selected.`);
}

function invertSelection() {
  if (!state.level) {
    return;
  }
  const current = new Set(selectionIndices());
  const inverted = state.level.props.map((_, index) => index).filter((index) => !current.has(index));
  setSelection(inverted, inverted.at(-1) ?? null);
  renderAll();
  updateViewportHint(`${inverted.length} props selected.`);
}

function renderSelectionSummary() {
  if (!el.selectionSummary) {
    return;
  }
  const count = selectionIndices().length;
  el.selectionSummary.textContent = `${count} selected`;
  el.deleteSelected.disabled = !state.level || count === 0;
}

function renderAvailabilityControls() {
  if (!el.toolSelect) {
    return;
  }
  const hasLevel = !!state.level && !state.levelLoading;
  const connected = !!state.connection.ready;
  const busy = state.projectLoading || state.levelLoading;
  const toolButtons = [
    ["select", el.toolSelect],
    ["move", el.toolMove],
    ["place", el.toolPlace],
    ["draw", el.toolDraw],
  ];

  if (!hasLevel && state.tool !== "select") {
    state.tool = "select";
  }
  for (const [tool, button] of toolButtons) {
    button.disabled = !hasLevel;
    button.classList.toggle("active", state.tool === tool);
  }

  el.refreshProject.disabled = state.projectLoading;
  el.newLevel.disabled = !connected || busy;
  el.duplicateLevel.disabled = !connected || !hasLevel || busy;
  el.validateLevel.disabled = !connected || !hasLevel || busy;
  el.saveLevel.disabled = !connected || !hasLevel || busy;
  el.deleteSelected.disabled = !hasLevel || selectionIndices().length === 0;
  el.createPrefab.disabled = !connected
    || !hasLevel
    || busy
    || selectionIndices().length === 0
    || !el.prefabName.value.trim()
    || !sanitizeLevelId(el.prefabId.value);
  el.deletePrefab.disabled = !connected || busy || state.prefab.loading || !state.prefab.selectedId;
}

function selectionCenter(indices = selectionIndices()) {
  if (indices.length === 0) {
    return [0, 0, 0];
  }
  const sum = indices.reduce(
    (total, index) => add3(total, vector(state.level.props[index].position)),
    [0, 0, 0]
  );
  return scale3(sum, 1 / indices.length);
}

function selectionToolsInspector() {
  const wrap = document.createElement("details");
  wrap.open = true;
  const indices = selectionIndices();
  wrap.innerHTML = `<summary>Transform (${indices.length})</summary>`;
  if (indices.length === 0) {
    const empty = document.createElement("div");
    empty.className = "inspector empty";
    empty.textContent = "Select props to transform them together.";
    wrap.append(empty);
    return wrap;
  }

  const tools = document.createElement("div");
  tools.className = "selection-tools";
  const header = document.createElement("div");
  header.className = "selection-tools-header";
  const axisLabel = state.transform.axis === "all" ? "XYZ" : state.transform.axis.toUpperCase();
  header.innerHTML = `<span>${indices.length === 1 ? "Selected prop" : `${indices.length} selected props`}</span><span>${escapeHtml(axisLabel)}</span>`;
  tools.append(header);
  const pivot = selectionCenter(indices);
  tools.append(vectorRow("Pivot", pivot, (value) => moveSelectionBy(sub3(value, selectionCenter()))));

  const nudges = document.createElement("div");
  nudges.className = "transform-grid";
  for (const [label, delta] of [
    ["X -", [-1, 0, 0]], ["X +", [1, 0, 0]],
    ["Y -", [0, -1, 0]], ["Y +", [0, 1, 0]],
    ["Z -", [0, 0, -1]], ["Z +", [0, 0, 1]],
  ]) {
    nudges.append(actionButton(label, () => nudgeSelection(delta)));
  }
  tools.append(nudges);

  const actions = document.createElement("div");
  actions.className = "inline-actions";
  actions.append(
    actionButton("Yaw -15", () => rotateSelection(1, -15)),
    actionButton("Yaw +15", () => rotateSelection(1, 15)),
    actionButton("Scale 0.5", () => scaleSelection(0.5)),
    actionButton("Scale 2", () => scaleSelection(2)),
    actionButton("Reset rotation", resetSelectionRotation),
    actionButton("Reset scale", resetSelectionScale)
  );
  tools.append(actions);
  wrap.append(tools);
  return wrap;
}

function renderObjects() {
  const props = state.level?.props || [];
  const filter = el.objectFilter.value.trim().toLowerCase();
  el.objectCount.textContent = String(props.length);
  el.objectList.innerHTML = "";
  props.forEach((prop, index) => {
    const label = propLabel(prop, index);
    if (filter && !label.toLowerCase().includes(filter)) {
      return;
    }
    const button = document.createElement("button");
    button.className = `object-card ${selectionContains(index) ? "active" : ""} ${
      index === state.selectedProp ? "primary" : ""
    }`;
    button.innerHTML = `<span class="card-title">${escapeHtml(label)}</span><span class="card-meta">${escapeHtml(prop.asset_id || "missing asset")} · ${kindForProp(prop)}</span>`;
    button.addEventListener("click", (event) => {
      selectProp(index, {
        additive: event.ctrlKey || event.metaKey,
        toggle: event.ctrlKey || event.metaKey,
        range: event.shiftKey,
      });
      state.workspaceTab = "props";
      renderAll();
    });
    el.objectList.append(button);
  });
}

function renderPalette() {
  const templates = templatesForProject();
  if (!state.selectedTemplate) {
    state.selectedTemplate = templates[0];
  }
  el.templateList.innerHTML = "";
  for (const template of templates) {
    const button = document.createElement("button");
    button.className = `template-card ${template.group} ${
      state.selectedTemplate === template ? "active" : ""
    }`;
    button.innerHTML = `<span class="card-title">${escapeHtml(template.label)}</span><span class="card-meta">${escapeHtml(template.asset_id)}</span>`;
    button.addEventListener("click", () => {
      clearSelectedPrefab();
      state.selectedTemplate = template;
      setTool("place");
      renderPalette();
      renderPrefabs();
    });
    el.templateList.append(button);
  }
  el.paletteStatus.textContent = state.selectedTemplate?.label || "Select a template";
}

function renderAssetBrowser() {
  if (!state.project || !el.assetList) {
    return;
  }
  const catalog = projectAssetCatalog();
  const kind = el.assetKindFilter.value;
  const filter = el.assetFilter.value.trim().toLowerCase();
  const assets = catalog.assets.filter((asset) => {
    const matchesKind = kind === "all" || asset.kind === kind;
    const haystack = `${asset.full_path} ${asset.filename} ${asset.kind} ${asset.format}`.toLowerCase();
    return matchesKind && (!filter || haystack.includes(filter));
  });
  el.assetCount.textContent = `${assets.length}/${catalog.assets.length}`;
  el.assetList.innerHTML = "";

  if (assets.length === 0) {
    const empty = document.createElement("div");
    empty.className = "inspector empty";
    empty.textContent = "No matching assets.";
    el.assetList.append(empty);
    return;
  }

  for (const asset of assets.slice(0, 80)) {
    const button = document.createElement("button");
    button.className = `asset-card ${asset.kind}`;
    const support = asset.runtime_supported ? "runtime" : "source";
    button.innerHTML = `<span class="card-title">${escapeHtml(asset.filename)}</span><span class="card-meta">${escapeHtml(asset.kind)} · ${escapeHtml(asset.format)} · ${support} · ${escapeHtml(asset.full_path)}</span>`;
    button.addEventListener("click", () => useAsset(asset));
    el.assetList.append(button);
  }
}

function renderPrefabs() {
  if (!el.prefabList) {
    return;
  }
  const prefabs = state.project?.prefabs || [];
  const filter = el.prefabFilter.value.trim().toLowerCase();
  const visible = prefabs.filter((prefab) => {
    const haystack = `${prefab.id} ${prefab.name}`.toLowerCase();
    return !filter || haystack.includes(filter);
  });
  el.prefabCount.textContent = `${visible.length}/${prefabs.length}`;
  el.prefabList.innerHTML = "";

  if (visible.length === 0) {
    const empty = document.createElement("div");
    empty.className = "inspector empty";
    empty.textContent = filter ? "No matching prefabs." : "No prefabs yet.";
    el.prefabList.append(empty);
  } else {
    for (const prefab of visible) {
      const button = document.createElement("button");
      button.className = `prefab-card ${state.prefab.selectedId === prefab.id ? "active" : ""}`;
      button.disabled = state.prefab.loading;
      button.innerHTML = `<span class="card-title">${escapeHtml(prefab.name || prefab.id)}</span><span class="card-meta">${escapeHtml(prefab.id)} · ${prefab.props} ${prefab.props === 1 ? "prop" : "props"}</span>`;
      button.addEventListener("click", () => loadPrefab(prefab.id));
      el.prefabList.append(button);
    }
  }

  if (state.prefab.data && state.prefab.selectedId) {
    el.prefabStatus.className = "inspector";
    el.prefabStatus.textContent = `${state.prefab.data.name} · ${state.prefab.data.props.length} ${state.prefab.data.props.length === 1 ? "prop" : "props"} · Ready to place`;
  } else if (state.prefab.loading) {
    el.prefabStatus.className = "inspector empty";
    el.prefabStatus.textContent = "Loading prefab...";
  } else {
    el.prefabStatus.className = "inspector empty";
    el.prefabStatus.textContent = "No prefab selected.";
  }
  el.deletePrefab.disabled = !state.connection.ready || !state.prefab.selectedId || state.prefab.loading;
}

async function loadPrefab(id) {
  if (!id || state.prefab.loading) {
    return false;
  }
  state.prefab.loading = true;
  renderPrefabs();
  try {
    const payload = await api(`/api/prefabs/${encodeURIComponent(id)}`);
    if (!payload.prefab || !Array.isArray(payload.prefab.props) || payload.prefab.props.length === 0) {
      throw new Error(`Prefab ${id} returned malformed editor data.`);
    }
    activatePrefab(id, payload.prefab);
    return true;
  } catch (error) {
    updateViewportHint(`Could not load prefab ${id}: ${error.message}`);
    return false;
  } finally {
    state.prefab.loading = false;
    renderPrefabs();
    renderAvailabilityControls();
  }
}

function activatePrefab(id, prefab) {
  state.prefab.selectedId = id;
  state.prefab.data = prefab;
  state.selectedTemplate = {
    group: "prefab",
    label: prefab.name || id,
    asset_id: `${prefab.props.length} ${prefab.props.length === 1 ? "prop" : "props"}`,
    prefab_id: id,
    prefab,
  };
  setTool("place");
  renderPalette();
  renderPrefabs();
  updateViewportHint(`${prefab.name || id} selected. Click any viewport to place the full prefab.`);
}

function clearSelectedPrefab() {
  const prefabTemplateSelected = state.selectedTemplate?.group === "prefab";
  state.prefab.selectedId = null;
  state.prefab.data = null;
  if (prefabTemplateSelected) {
    state.selectedTemplate = null;
  }
}

async function createPrefabFromSelection() {
  const indices = selectionIndices();
  if (!state.connection.ready || !state.level || indices.length === 0) {
    updateViewportHint("Select one or more props before creating a prefab.");
    return false;
  }
  const name = el.prefabName.value.trim();
  if (!name) {
    updateViewportHint("Enter a prefab name before creating it.");
    el.prefabName.focus();
    return false;
  }
  const id = sanitizeLevelId(el.prefabId.value);
  if (!id) {
    updateViewportHint("Enter a safe prefab id using letters, numbers, hyphens, or underscores.");
    el.prefabId.focus();
    return false;
  }
  el.prefabId.value = id;
  if ((state.project.prefabs || []).some((prefab) => prefab.id === id)
      && !confirm(`Overwrite prefab '${id}'? A backup will be created.`)) {
    return false;
  }

  const bounds = selectionBounds(indices);
  const origin = [bounds.center[0], bounds.min[1], bounds.center[2]];
  let prefab;
  try {
    prefab = window.CenotaphPrefabTools.fromSelection(
      name,
      indices.map((index) => state.level.props[index]),
      origin
    );
    await api(`/api/prefabs/${encodeURIComponent(id)}`, {
      method: "PUT",
      body: JSON.stringify(prefab),
    });
    await refreshProject();
    activatePrefab(id, prefab);
    setWorkspaceTab("prefabs");
    el.prefabName.value = "";
    el.prefabId.value = "";
    el.prefabId.dataset.manual = "false";
    updateViewportHint(`Created prefab ${name} from ${indices.length} ${indices.length === 1 ? "prop" : "props"}.`);
    return true;
  } catch (error) {
    updateViewportHint(`Prefab was not created: ${error.message}`);
    return false;
  }
}

async function deleteSelectedPrefab() {
  const id = state.prefab.selectedId;
  if (!id || state.prefab.loading) {
    return false;
  }
  const name = state.prefab.data?.name || id;
  if (!confirm(`Delete prefab '${name}'? A backup will be kept.`)) {
    return false;
  }
  state.prefab.loading = true;
  renderPrefabs();
  try {
    await api(`/api/prefabs/${encodeURIComponent(id)}`, { method: "DELETE" });
    clearSelectedPrefab();
    await refreshProject();
    renderAll();
    updateViewportHint(`Deleted prefab ${name}. Its backup remains in prefabs/.editor_backups/.`);
    return true;
  } catch (error) {
    updateViewportHint(`Prefab was not deleted: ${error.message}`);
    return false;
  } finally {
    state.prefab.loading = false;
    renderPrefabs();
    renderAvailabilityControls();
  }
}

function renderKeybindings() {
  if (!el.keybindingList) {
    return;
  }
  el.keybindingList.innerHTML = "";
  for (const action of Object.keys(defaultKeybindings)) {
    const binding = state.keybindings[action];
    const rowNode = document.createElement("div");
    rowNode.className = `keybinding-row ${state.capturingBinding === action ? "capturing" : ""}`;
    const label = document.createElement("span");
    label.textContent = defaultKeybindings[action].label;
    const button = document.createElement("button");
    button.textContent = state.capturingBinding === action ? "Press key" : binding.display;
    button.addEventListener("click", () => {
      state.capturingBinding = action;
      renderKeybindings();
      updateViewportHint(`Press a key for ${defaultKeybindings[action].label}, or Esc to cancel.`);
    });
    rowNode.append(label, button);
    el.keybindingList.append(rowNode);
  }
}

function useAsset(asset) {
  if (asset.kind === "model" && asset.runtime_supported && asset.root_path === "assets") {
    clearSelectedPrefab();
    state.selectedTemplate = {
      group: "geometry",
      label: asset.filename,
      asset_id: asset.relative_path,
      scale: [1, 1, 1],
      collider_type: "Box",
    };
    setTool("place");
    renderPalette();
    renderPrefabs();
    updateViewportHint(`Model selected for placement: ${asset.relative_path}`);
    return;
  }

  stageAssetImport(asset);
}

function stageAssetImport(asset) {
  if (!state.level) {
    updateViewportHint("Load or create a level before staging assets.");
    return;
  }
  state.level.asset_imports = state.level.asset_imports || [];
  const id = uniqueImportId(slugFromPath(asset.filename || asset.relative_path));
  const asset_id = asset.root_path === "assets" ? asset.relative_path : asset.full_path;
  state.level.asset_imports.push({
    id,
    asset_id,
    source_path: asset.full_path,
    default_scale: [1, 1, 1],
    default_collider_type: asset.kind === "model" && asset.runtime_supported ? "Box" : "None",
    tags: [asset.kind, asset.format],
    notes: asset.runtime_supported ? "runtime-supported" : "source-only",
  });
  markDirty();
  renderSystemPanels();
  updateViewportHint(`Staged ${asset.kind} import ${id}. Validate before saving.`);
}

function templatesForProject() {
  const relicTemplates = (state.project?.relics || []).map((relic) => ({
    group: "item",
    label: relic.display_name || relic.id,
    asset_id: pickupAssetForRelic(relic.id),
    scale: [0.35, 0.35, 0.35],
    item_id: relic.id,
  }));
  const enemyTemplates = (state.project?.enemies || []).map((enemy) => ({
    group: "enemy",
    label: enemy.display_name || enemy.id,
    asset_id: enemy.model_asset,
    scale: enemyScale(enemy.id),
    collider_type: enemy.collider_type || "Sphere",
    enemy_type: enemy.id,
    enemy_health: enemy.health || 1,
  }));
  return [...baseTemplates, ...relicTemplates, ...enemyTemplates];
}

function renderInspector() {
  if (!state.level) {
    el.inspector.className = "inspector empty";
    el.inspector.textContent = "Load or create a level.";
    return;
  }
  el.inspector.className = "inspector";
  el.inspector.innerHTML = "";
  el.inspector.append(levelInspector());
  el.inspector.append(selectionToolsInspector());
  if (state.selectedProp != null && state.level.props[state.selectedProp]) {
    el.inspector.append(propInspector(state.level.props[state.selectedProp], state.selectedProp));
  } else {
    const empty = document.createElement("div");
    empty.className = "inspector empty";
    empty.textContent = "Select a prop in the viewport or object list.";
    el.inspector.append(empty);
  }
}

function levelInspector() {
  const wrap = document.createElement("details");
  wrap.open = true;
  wrap.innerHTML = "<summary>Level</summary>";
  const grid = document.createElement("div");
  grid.className = "form-grid";
  grid.append(textRow("Name", state.level.name || "", (value) => setLevelValue("name", value)));
  grid.append(
    selectRow(
      "Base map",
      state.level.base_map || "",
      assetOptions(["obj", "glb", "gltf"], true),
      (value) => setLevelValue("base_map", value)
    )
  );
  grid.append(vectorRow("Spawn", state.level.player_spawn, (value) => setLevelValue("player_spawn", value)));
  wrap.append(grid);
  return wrap;
}

function propInspector(prop, index) {
  const wrap = document.createElement("details");
  wrap.open = true;
  const label = selectionIndices().length > 1 ? "Primary Prop" : "Selected Prop";
  wrap.innerHTML = `<summary>${label} ${index + 1}</summary>`;
  const grid = document.createElement("div");
  grid.className = "form-grid";
  grid.append(textRow("Id", prop.id || "", (value) => setPropValue(prop, "id", nullable(value))));
  grid.append(
    selectRow("Asset", prop.asset_id || "", assetOptions(["obj", "glb", "gltf"], false), (value) =>
      setPropValue(prop, "asset_id", value)
    )
  );
  grid.append(vectorRow("Position", prop.position, (value) => setPropValue(prop, "position", value)));
  grid.append(vectorRow("Rotation", prop.rotation, (value) => setPropValue(prop, "rotation", value)));
  grid.append(vectorRow("Scale", prop.scale, (value) => setPropValue(prop, "scale", value)));
  grid.append(
    selectRow("Collider", prop.collider_type || "None", ["None", "Box", "Sphere", "Mesh"], (value) =>
      setPropValue(prop, "collider_type", value)
    )
  );
  grid.append(checkRow("Climbable", !!prop.is_climbable, (value) => setPropValue(prop, "is_climbable", value)));
  grid.append(checkRow("Hurtbox", !!prop.is_hurtbox, (value) => setPropValue(prop, "is_hurtbox", value)));
  grid.append(selectRow("Item", prop.item_id || "", ["", ...relicIds()], (value) => setPropValue(prop, "item_id", nullable(value))));
  grid.append(numberRow("Resource", prop.resource_value || 0, (value) => setPropValue(prop, "resource_value", value)));
  grid.append(selectRow("Enemy", prop.enemy_type || "", ["", ...enemyIds()], (value) => setPropValue(prop, "enemy_type", nullable(value))));
  grid.append(numberRow("Health", prop.enemy_health || 0, (value) => setPropValue(prop, "enemy_health", value)));
  grid.append(textRow("Anchor", prop.anchor_id || "", (value) => setPropValue(prop, "anchor_id", nullable(value))));
  grid.append(textRow("Gate level", prop.trigger_level_id || "", (value) => setPropValue(prop, "trigger_level_id", nullable(value))));
  grid.append(textRow("Loot table", prop.loot_table_id || "", (value) => setPropValue(prop, "loot_table_id", nullable(value))));
  grid.append(textRow("Path", prop.path_id || "", (value) => setPropValue(prop, "path_id", nullable(value))));
  grid.append(textRow("Dialogue", prop.dialogue_id || "", (value) => setPropValue(prop, "dialogue_id", nullable(value))));
  grid.append(textRow("Event", prop.event_id || "", (value) => setPropValue(prop, "event_id", nullable(value))));
  wrap.append(grid);
  wrap.append(brushGeometryInspector(prop));
  return wrap;
}

function brushGeometryInspector(prop) {
  const wrap = document.createElement("details");
  wrap.open = !!prop.brush_geometry;
  const geometryKind = prop.brush_geometry?.kind === "terrain" ? "Terrain" : "Brush";
  wrap.innerHTML = `<summary>${geometryKind} Geometry</summary>`;

  const actions = document.createElement("div");
  actions.className = "inline-actions brush-actions";
  actions.append(
    actionButton("Convert", () => convertPropToBrush(prop, "box")),
    actionButton("Slope +X", () => convertPropToBrush(prop, "slope", "x+")),
    actionButton("Slope -X", () => convertPropToBrush(prop, "slope", "x-")),
    actionButton("Slope +Z", () => convertPropToBrush(prop, "slope", "z+")),
    actionButton("Slope -Z", () => convertPropToBrush(prop, "slope", "z-")),
    actionButton("Cylinder", () => convertPropToBrush(prop, "cylinder")),
    actionButton("Stairs", () => convertPropToBrush(prop, "stairs", state.drawBrush.direction)),
    actionButton("Terrain", () => convertPropToBrush(prop, "terrain"))
  );
  wrap.append(actions);

  if (!prop.brush_geometry) {
    const empty = document.createElement("div");
    empty.className = "inspector empty";
    empty.textContent = "Convert this prop to edit vertices, slopes, and custom brush faces.";
    wrap.append(empty);
    return wrap;
  }

  normalizeBrushGeometry(prop.brush_geometry);
  if (prop.brush_geometry.kind === "terrain") {
    wrap.append(terrainSculptInspector(prop));
  }
  const editActions = document.createElement("div");
  editActions.className = "inline-actions brush-actions";
  editActions.append(
    actionButton("Flatten Top", () => flattenBrushTop(prop)),
    actionButton("Mirror X", () => mirrorBrush(prop, 0)),
    actionButton("Mirror Z", () => mirrorBrush(prop, 2)),
    actionButton("Snap", () => snapBrushVertices(prop)),
    actionButton("Recenter", () => recenterBrushGeometry(prop)),
    actionButton("Remove Mesh", () => removeBrushGeometry(prop))
  );
  wrap.append(editActions);

  const rawGeometry = document.createElement("details");
  rawGeometry.className = "raw-geometry";
  rawGeometry.open = prop.brush_geometry.vertices.length <= 32;
  rawGeometry.innerHTML = `<summary>Raw mesh (${prop.brush_geometry.vertices.length} vertices)</summary>`;

  const vertexGrid = document.createElement("div");
  vertexGrid.className = "vertex-grid";
  prop.brush_geometry.vertices.forEach((vertex, vertexIndex) => {
    const rowNode = document.createElement("div");
    rowNode.className = "vertex-row";
    const label = document.createElement("span");
    label.textContent = `${vertexIndex} ${brushVertexLabels[vertexIndex] || "vertex"}`;
    rowNode.append(label);
    for (let axis = 0; axis < 3; axis += 1) {
      const input = document.createElement("input");
      input.className = "field";
      input.type = "number";
      input.step = "0.1";
      input.value = formatNumber(vertex[axis]);
      input.addEventListener("input", () => {
        const parsed = Number(input.value);
        if (!Number.isFinite(parsed)) {
          return;
        }
        prop.brush_geometry.vertices[vertexIndex][axis] = parsed;
        markDirty();
        renderObjects();
        updateViewportHint(`Edited vertex ${vertexIndex}.`);
        draw();
      });
      rowNode.append(input);
    }
    vertexGrid.append(rowNode);
  });
  rawGeometry.append(vertexGrid);

  const facesArea = document.createElement("textarea");
  facesArea.className = "face-editor";
  facesArea.value = JSON.stringify(prop.brush_geometry.faces, null, 2);
  facesArea.addEventListener("change", () => {
    try {
      const faces = JSON.parse(facesArea.value || "[]");
      if (!Array.isArray(faces)) {
        throw new Error("faces must be an array");
      }
      prop.brush_geometry.faces = faces
        .map((face) => Array.isArray(face) ? face.slice(0, 3).map((value) => Math.max(0, Math.floor(Number(value) || 0))) : null)
        .filter((face) => face && face.length === 3);
      normalizeBrushGeometry(prop.brush_geometry);
      facesArea.style.borderColor = "";
      markDirty();
      renderInspector();
      draw();
    } catch (_) {
      facesArea.style.borderColor = "var(--danger)";
    }
  });
  rawGeometry.append(row("Faces", facesArea));
  wrap.append(rawGeometry);
  return wrap;
}

function terrainSculptInspector(prop) {
  const geometry = prop.brush_geometry;
  const config = terrainConfig(geometry);
  const panel = document.createElement("div");
  panel.className = "terrain-sculpt";

  const header = document.createElement("div");
  header.className = "selection-tools-header";
  header.textContent = `${config.columns} × ${config.rows} Terrain`;
  panel.append(header);

  const controls = document.createElement("div");
  controls.className = "form-grid terrain-controls";
  controls.append(
    numberRow("Strength", config.sculpt_strength, (value) => {
      config.sculpt_strength = clamp(value, 0.05, 64);
      markDirty();
    }),
    numberRow("Seed (regen)", config.seed, (value) => {
      config.seed = Math.max(0, Math.round(value));
      markDirty();
    })
  );
  panel.append(controls);

  const actions = document.createElement("div");
  actions.className = "inline-actions brush-actions";
  actions.append(
    actionButton("Raise Center", () => sculptTerrain(prop, "raise")),
    actionButton("Lower Center", () => sculptTerrain(prop, "lower")),
    actionButton("Smooth", () => sculptTerrain(prop, "smooth")),
    actionButton("Flatten", () => sculptTerrain(prop, "flatten")),
    actionButton("Regenerate", () => regenerateTerrain(prop, false)),
    actionButton("New Seed", () => regenerateTerrain(prop, true))
  );
  panel.append(actions);
  return panel;
}

function terrainConfig(geometry) {
  const vertexGuess = Math.max(2, Math.round(Math.sqrt(Math.max(2, geometry.vertices.length * 0.5))) - 1);
  geometry.kind = "terrain";
  geometry.terrain = geometry.terrain || {};
  geometry.terrain.columns = Math.round(clamp(numberValue(geometry.terrain.columns) || vertexGuess, 2, 24));
  geometry.terrain.rows = Math.round(clamp(numberValue(geometry.terrain.rows) || vertexGuess, 2, 24));
  geometry.terrain.seed = Math.max(0, Math.round(numberValue(geometry.terrain.seed)));
  geometry.terrain.relief = clamp(numberValue(geometry.terrain.relief), 0, 256);
  geometry.terrain.base_thickness = clamp(numberValue(geometry.terrain.base_thickness) || 0.5, 0.05, 256);
  geometry.terrain.sculpt_strength = clamp(numberValue(geometry.terrain.sculpt_strength) || 0.5, 0.05, 64);
  return geometry.terrain;
}

function terrainGridInfo(geometry) {
  const config = terrainConfig(geometry);
  const topVertexCount = (config.columns + 1) * (config.rows + 1);
  if (geometry.vertices.length < topVertexCount * 2) {
    return null;
  }
  return { config, topVertexCount };
}

function sculptTerrain(prop, operation) {
  const geometry = normalizeBrushGeometry(prop.brush_geometry);
  const grid = terrainGridInfo(geometry);
  if (!grid) {
    updateViewportHint("Terrain sculpting is unavailable because the terrain grid metadata does not match its mesh.");
    return;
  }

  const { config, topVertexCount } = grid;
  const top = geometry.vertices.slice(0, topVertexCount);
  const bottom = geometry.vertices.slice(topVertexCount, topVertexCount * 2);
  const bottomY = Math.min(...bottom.map((vertex) => vertex[1]));
  const strength = config.sculpt_strength;

  if (operation === "raise" || operation === "lower") {
    const bounds = localBrushBounds({ vertices: top, faces: [] });
    const halfWidth = Math.max(0.1, (bounds.max[0] - bounds.min[0]) * 0.5);
    const halfDepth = Math.max(0.1, (bounds.max[2] - bounds.min[2]) * 0.5);
    const centerX = (bounds.min[0] + bounds.max[0]) * 0.5;
    const centerZ = (bounds.min[2] + bounds.max[2]) * 0.5;
    const direction = operation === "raise" ? 1 : -1;
    for (const vertex of top) {
      const distance = Math.hypot((vertex[0] - centerX) / halfWidth, (vertex[2] - centerZ) / halfDepth);
      const weight = smoothStep(1, 0, clamp(distance, 0, 1));
      vertex[1] = Math.max(bottomY + 0.05, vertex[1] + direction * strength * weight);
    }
  } else if (operation === "smooth") {
    const original = top.map((vertex) => vertex[1]);
    const blend = clamp(strength * 0.2, 0.1, 0.85);
    for (let rowIndex = 0; rowIndex <= config.rows; rowIndex += 1) {
      for (let columnIndex = 0; columnIndex <= config.columns; columnIndex += 1) {
        let total = 0;
        let count = 0;
        for (let rowOffset = -1; rowOffset <= 1; rowOffset += 1) {
          for (let columnOffset = -1; columnOffset <= 1; columnOffset += 1) {
            const row = rowIndex + rowOffset;
            const column = columnIndex + columnOffset;
            if (row < 0 || row > config.rows || column < 0 || column > config.columns) continue;
            total += original[row * (config.columns + 1) + column];
            count += 1;
          }
        }
        const index = rowIndex * (config.columns + 1) + columnIndex;
        top[index][1] = lerp(original[index], total / count, blend);
      }
    }
  } else if (operation === "flatten") {
    const targetY = top.reduce((total, vertex) => total + vertex[1], 0) / top.length;
    for (const vertex of top) {
      vertex[1] = targetY;
    }
  }

  const topHeights = top.map((vertex) => vertex[1]);
  config.relief = Math.max(...topHeights) - Math.min(...topHeights);
  markDirty();
  renderInspector();
  renderObjects();
  draw();
  updateViewportHint(`${titleCase(operation)} terrain completed.`);
}

function regenerateTerrain(prop, advanceSeed) {
  const geometry = normalizeBrushGeometry(prop.brush_geometry);
  const config = terrainConfig(geometry);
  if (advanceSeed) {
    config.seed += 1;
  }
  const oldBounds = localBrushBounds(geometry);
  const width = Math.max(0.1, oldBounds.max[0] - oldBounds.min[0]);
  const depth = Math.max(0.1, oldBounds.max[2] - oldBounds.min[2]);
  const height = Math.max(0.1, config.base_thickness + config.relief);
  const next = terrainBrushGeometry(
    [width, height, depth],
    Math.max(config.columns, config.rows),
    config.seed,
    config.relief,
    config.base_thickness
  );
  next.terrain.sculpt_strength = config.sculpt_strength;
  const nextBounds = localBrushBounds(next);
  const localOffset = [0, oldBounds.min[1] - nextBounds.min[1], 0];
  prop.position = add3(vector(prop.position), transformLocalOffset(prop, localOffset));
  prop.brush_geometry = next;
  markDirty();
  renderAll();
  updateViewportHint(advanceSeed ? `Terrain regenerated with seed ${next.terrain.seed}.` : "Terrain regenerated.");
}

function actionButton(label, onClick) {
  const button = document.createElement("button");
  button.className = "small";
  button.type = "button";
  button.textContent = label;
  button.addEventListener("click", onClick);
  return button;
}

function graphInspector() {
  const wrap = document.createElement("details");
  wrap.innerHTML = "<summary>Level Systems</summary>";
  wrap.append(
    systemSection("Asset Imports", "asset_imports", addAssetImportStub, assetImportCard),
    systemSection("Loot Tables", "loot_tables", addLootTable, lootTableCard),
    systemSection("Paths", "paths", addPath, pathCard),
    systemSection("Events", "events", addEvent, eventCard),
    systemSection("Dialogues", "dialogues", addDialogue, dialogueCard)
  );
  return wrap;
}

function systemSection(title, key, addFn, cardFn) {
  state.level[key] = state.level[key] || [];
  const section = document.createElement("div");
  section.className = "system-section";
  const header = document.createElement("div");
  header.className = "system-header";
  const label = document.createElement("span");
  label.textContent = `${title} (${state.level[key].length})`;
  header.append(label, actionButton("Add", () => addFn()));
  section.append(header);

  if (state.level[key].length === 0) {
    const empty = document.createElement("div");
    empty.className = "inspector empty";
    empty.textContent = "None authored yet.";
    section.append(empty);
    return section;
  }

  state.level[key].forEach((item, index) => section.append(cardFn(item, index)));
  return section;
}

function systemCard(summary, key, item, index, buildFields) {
  const card = document.createElement("details");
  card.className = "system-card";
  card.innerHTML = `<summary>${escapeHtml(summary)}</summary>`;

  const actions = document.createElement("div");
  actions.className = "inline-actions system-actions";
  actions.append(
    actionButton("Duplicate", () => duplicateSystemItem(key, index)),
    actionButton("Remove", () => removeSystemItem(key, index))
  );
  card.append(actions);

  const grid = document.createElement("div");
  grid.className = "form-grid";
  buildFields(grid);
  card.append(grid);

  const raw = document.createElement("textarea");
  raw.className = "system-json";
  raw.value = JSON.stringify(item, null, 2);
  raw.addEventListener("change", () => {
    try {
      state.level[key][index] = JSON.parse(raw.value || "{}");
      raw.style.borderColor = "";
      markDirty();
      renderSystemPanels();
      draw();
    } catch (_) {
      raw.style.borderColor = "var(--danger)";
    }
  });
  card.append(row("Raw", raw));
  return card;
}

function assetImportCard(item, index) {
  return systemCard(item.id || `asset import ${index + 1}`, "asset_imports", item, index, (grid) => {
    grid.append(textRow("Id", item.id || "", (value) => setSystemValue(item, "id", sanitizeAuthoringId(value))));
    grid.append(selectRow("Asset", item.asset_id || "", assetOptions(["obj", "glb", "gltf"], false), (value) => setSystemValue(item, "asset_id", value)));
    grid.append(textRow("Source", item.source_path || "", (value) => setSystemValue(item, "source_path", nullable(value))));
    grid.append(vectorRow("Scale", item.default_scale || [1, 1, 1], (value) => setSystemValue(item, "default_scale", value)));
    grid.append(selectRow("Collider", item.default_collider_type || "None", ["None", "Box", "Sphere", "Mesh"], (value) => setSystemValue(item, "default_collider_type", value)));
    grid.append(textRow("Tags", (item.tags || []).join(", "), (value) => setSystemValue(item, "tags", csvList(value))));
    grid.append(textRow("Notes", item.notes || "", (value) => setSystemValue(item, "notes", nullable(value))));
  });
}

function lootTableCard(item, index) {
  return systemCard(item.id || `loot table ${index + 1}`, "loot_tables", item, index, (grid) => {
    grid.append(textRow("Id", item.id || "", (value) => setSystemValue(item, "id", sanitizeAuthoringId(value))));
    grid.append(numberRow("Rolls", item.rolls || 1, (value) => setSystemValue(item, "rolls", Math.max(1, Math.floor(value)))));
    grid.append(row("Entries", lootEntriesEditor(item)));
    grid.append(jsonRow("Entries", item.entries || [], (value) => setSystemValue(item, "entries", value, true)));
  });
}

function pathCard(item, index) {
  return systemCard(item.id || `path ${index + 1}`, "paths", item, index, (grid) => {
    grid.append(textRow("Id", item.id || "", (value) => setSystemValue(item, "id", sanitizeAuthoringId(value))));
    grid.append(selectRow("Kind", item.kind || "Enemy", ["Enemy", "Npc", "Platform", "Cinematic"], (value) => setSystemValue(item, "kind", value)));
    grid.append(checkRow("Looped", !!item.looped, (value) => setSystemValue(item, "looped", value)));
    grid.append(numberRow("Speed", item.speed_multiplier || 1, (value) => setSystemValue(item, "speed_multiplier", Math.max(0.1, value))));
    grid.append(row("Waypoints", waypointEditor(item)));
    grid.append(jsonRow("Waypoints", item.waypoints || [], (value) => setSystemValue(item, "waypoints", value, true)));
  });
}

function eventCard(item, index) {
  item.trigger = item.trigger || {};
  return systemCard(item.id || `event ${index + 1}`, "events", item, index, (grid) => {
    grid.append(textRow("Id", item.id || "", (value) => setSystemValue(item, "id", sanitizeAuthoringId(value))));
    grid.append(checkRow("Once", item.once !== false, (value) => setSystemValue(item, "once", value)));
    grid.append(selectRow("Trigger", item.trigger.kind || "Proximity", ["Proximity", "OnEnter", "Interact", "Manual"], (value) => setTriggerValue(item, "kind", value)));
    grid.append(vectorRow("Position", item.trigger.position || [0, 0, 0], (value) => setTriggerValue(item, "position", value)));
    grid.append(numberRow("Radius", item.trigger.radius || 2.5, (value) => setTriggerValue(item, "radius", Math.max(0.1, value))));
    grid.append(selectRow("Prop", item.trigger.prop_id || "", propIdOptions(), (value) => setTriggerValue(item, "prop_id", nullable(value))));
    grid.append(textRow("Flag", item.trigger.flag_id || "", (value) => setTriggerValue(item, "flag_id", nullable(value))));
    grid.append(row("Actions", eventActionsEditor(item)));
    grid.append(jsonRow("Actions", item.actions || [], (value) => setSystemValue(item, "actions", value, true)));
  });
}

function dialogueCard(item, index) {
  return systemCard(item.id || `dialogue ${index + 1}`, "dialogues", item, index, (grid) => {
    grid.append(textRow("Id", item.id || "", (value) => setSystemValue(item, "id", sanitizeAuthoringId(value))));
    grid.append(textRow("Speaker", item.speaker || "", (value) => setSystemValue(item, "speaker", value)));
    grid.append(linesRow("Lines", item.lines || [], (value) => setSystemValue(item, "lines", value, true)));
  });
}

function lootEntriesEditor(table) {
  table.entries = table.entries || [];
  const wrap = nestedList("No loot entries yet.", actionButton("Add Entry", () => addLootEntry(table)), table.entries.length === 0);
  table.entries.forEach((entry, index) => {
    const card = nestedCard(`Entry ${index + 1}`, [
      actionButton("Duplicate", () => duplicateNestedItem(table.entries, index)),
      actionButton("Remove", () => removeNestedItem(table.entries, index)),
    ]);
    const grid = document.createElement("div");
    grid.className = "form-grid";
    grid.append(
      numberRow("Weight", entry.weight || 1, (value) => setNestedValue(entry, "weight", Math.max(1, Math.floor(value)))),
      textRow("Item", entry.item_id || "", (value) => setNestedValue(entry, "item_id", nullable(value))),
      numberRow("Resource", entry.resource_value || 0, (value) => setNestedValue(entry, "resource_value", Math.max(0, Math.floor(value)))),
      numberRow("Quantity", entry.quantity || 1, (value) => setNestedValue(entry, "quantity", Math.max(1, Math.floor(value))))
    );
    card.append(grid);
    wrap.append(card);
  });
  return wrap;
}

function waypointEditor(path) {
  path.waypoints = path.waypoints || [];
  const wrap = nestedList("No waypoints yet.", actionButton("Add Point", () => addWaypoint(path)), path.waypoints.length === 0);
  path.waypoints.forEach((point, index) => {
    const card = nestedCard(`Point ${index + 1}`, [
      actionButton("Duplicate", () => duplicateNestedItem(path.waypoints, index)),
      actionButton("Remove", () => removeNestedItem(path.waypoints, index)),
    ]);
    card.append(vectorRow("Position", point, (value) => setWaypoint(path, index, value)));
    wrap.append(card);
  });
  return wrap;
}

function eventActionsEditor(eventItem) {
  eventItem.actions = eventItem.actions || [];
  const wrap = nestedList("No actions yet.", actionButton("Add Action", () => addEventAction(eventItem)), eventItem.actions.length === 0);
  eventItem.actions.forEach((action, index) => {
    const card = nestedCard(action.kind || `Action ${index + 1}`, [
      actionButton("Duplicate", () => duplicateNestedItem(eventItem.actions, index)),
      actionButton("Remove", () => removeNestedItem(eventItem.actions, index)),
    ]);
    const grid = document.createElement("div");
    grid.className = "form-grid";
    const kind = action.kind || "SetFlag";
    grid.append(selectRow("Kind", kind, ["SetFlag", "LoadLevel", "GrantResource", "SpawnLoot", "StartDialogue"], (value) => resetEventActionKind(action, value, eventItem)));
    if (kind === "SetFlag") {
      grid.append(textRow("Flag", action.flag_id || "", (value) => setNestedValue(action, "flag_id", nullable(value))));
    } else if (kind === "LoadLevel") {
      grid.append(selectRow("Target", action.target_level_id || "", levelOptions(), (value) => setNestedValue(action, "target_level_id", nullable(value))));
    } else if (kind === "GrantResource") {
      grid.append(numberRow("Resource", action.resource_value || 0, (value) => setNestedValue(action, "resource_value", Math.max(0, Math.floor(value)))));
    } else if (kind === "SpawnLoot") {
      grid.append(selectRow("Loot Table", action.loot_table_id || "", lootTableOptions(), (value) => setNestedValue(action, "loot_table_id", nullable(value))));
      grid.append(vectorRow("Spawn", action.spawn_position || vector(state.level.player_spawn), (value) => setNestedValue(action, "spawn_position", value)));
    } else if (kind === "StartDialogue") {
      grid.append(selectRow("Dialogue", action.dialogue_id || "", dialogueOptions(), (value) => setNestedValue(action, "dialogue_id", nullable(value))));
    }
    card.append(grid);
    wrap.append(card);
  });
  return wrap;
}

function nestedList(emptyText, addButton, isEmpty) {
  const wrap = document.createElement("div");
  wrap.className = "nested-list";
  const actions = document.createElement("div");
  actions.className = "inline-actions nested-actions";
  actions.append(addButton);
  wrap.append(actions);
  const empty = document.createElement("div");
  empty.className = "inspector empty nested-empty";
  empty.textContent = emptyText;
  empty.hidden = !isEmpty;
  wrap.append(empty);
  return wrap;
}

function nestedCard(title, actions) {
  const card = document.createElement("div");
  card.className = "nested-card";
  const header = document.createElement("div");
  header.className = "nested-header";
  const label = document.createElement("span");
  label.textContent = title;
  const actionWrap = document.createElement("div");
  actionWrap.className = "inline-actions";
  actionWrap.append(...actions);
  header.append(label, actionWrap);
  card.append(header);
  return card;
}

function addLootEntry(table) {
  table.entries = table.entries || [];
  table.entries.push({ weight: 1, resource_value: 25, quantity: 1 });
  markDirty();
  renderSystemPanels();
  draw();
}

function addWaypoint(path) {
  path.waypoints = path.waypoints || [];
  const last = vector(path.waypoints[path.waypoints.length - 1] || state.level.player_spawn);
  path.waypoints.push([snap(last[0] + 4), last[1], snap(last[2] + 4)]);
  markDirty();
  renderSystemPanels();
  draw();
}

function addEventAction(eventItem) {
  eventItem.actions = eventItem.actions || [];
  eventItem.actions.push(eventActionDefaults("SetFlag", eventItem.id));
  markDirty();
  renderSystemPanels();
  draw();
}

function setWaypoint(path, index, value) {
  path.waypoints[index] = vector(value);
  markDirty();
  draw();
}

function setNestedValue(item, key, value, rerender = false) {
  item[key] = value;
  markDirty();
  if (rerender) renderSystemPanels();
  draw();
}

function duplicateNestedItem(list, index) {
  const item = list[index];
  if (!item) return;
  list.splice(index + 1, 0, JSON.parse(JSON.stringify(item)));
  markDirty();
  renderSystemPanels();
  draw();
}

function removeNestedItem(list, index) {
  list.splice(index, 1);
  markDirty();
  renderSystemPanels();
  draw();
}

function resetEventActionKind(action, kind, eventItem) {
  for (const key of ["target_level_id", "loot_table_id", "dialogue_id", "flag_id", "resource_value", "spawn_position"]) {
    delete action[key];
  }
  Object.assign(action, eventActionDefaults(kind, eventItem.id));
  markDirty();
  renderSystemPanels();
  draw();
}

function eventActionDefaults(kind, eventId = "event") {
  if (kind === "LoadLevel") {
    return { kind, target_level_id: firstOption(levelOptions()) };
  }
  if (kind === "GrantResource") {
    return { kind, resource_value: 25 };
  }
  if (kind === "SpawnLoot") {
    return { kind, loot_table_id: firstOption(lootTableOptions()), spawn_position: vector(state.level.player_spawn) };
  }
  if (kind === "StartDialogue") {
    return { kind, dialogue_id: firstOption(dialogueOptions()) };
  }
  const flagBase = sanitizeAuthoringId(eventId) || "event";
  return { kind: "SetFlag", flag_id: `${flagBase}_done` };
}

function firstOption(options) {
  return options.find((option) => option) || null;
}

function levelOptions() {
  return ["", ...(state.project?.levels || []).map((level) => level.id)];
}

function lootTableOptions() {
  return ["", ...(state.level?.loot_tables || []).map((table) => table.id).filter(Boolean)];
}

function dialogueOptions() {
  return ["", ...(state.level?.dialogues || []).map((dialogue) => dialogue.id).filter(Boolean)];
}

function propIdOptions() {
  return ["", ...(state.level?.props || []).map((prop) => prop.id).filter(Boolean)];
}

function addAssetImportStub() {
  const asset = assetOptions(["obj", "glb", "gltf"], false)[0] || "props/test_wall.obj";
  pushSystemItem("asset_imports", {
    id: uniqueSystemId("asset_imports", "import"),
    asset_id: asset,
    source_path: null,
    default_scale: [1, 1, 1],
    default_collider_type: "Box",
    tags: ["model"],
    notes: null,
  });
}

function addLootTable() {
  pushSystemItem("loot_tables", {
    id: uniqueSystemId("loot_tables", "loot"),
    rolls: 1,
    entries: [{ weight: 1, resource_value: 25, quantity: 1 }],
  });
}

function addPath() {
  const spawn = vector(state.level.player_spawn);
  pushSystemItem("paths", {
    id: uniqueSystemId("paths", "path"),
    kind: "Enemy",
    looped: false,
    speed_multiplier: 1,
    waypoints: [spawn, [spawn[0] + 6, spawn[1], spawn[2] + 6]],
  });
}

function addPathPointAt(world) {
  if (!state.level || !world) {
    return;
  }
  state.level.paths = state.level.paths || [];
  if (state.level.paths.length === 0) {
    addPath();
  }
  const path = state.level.paths[state.level.paths.length - 1];
  path.waypoints = path.waypoints || [];
  path.waypoints.push([snap(world.x), world.y ?? defaultPlacementY(), snap(world.z)]);
  markDirty();
  renderSystemPanels();
  updateViewportHint(`Added waypoint to ${path.id}.`);
  draw();
}

function addEvent() {
  const spawn = vector(state.level.player_spawn);
  const id = uniqueSystemId("events", "event");
  pushSystemItem("events", {
    id,
    once: true,
    trigger: { kind: "Proximity", position: spawn, radius: 3 },
    actions: [{ kind: "SetFlag", flag_id: `${id}_done` }],
  });
}

function addEventTriggerAt(world) {
  if (!state.level || !world) {
    return;
  }
  const id = uniqueSystemId("events", "event");
  pushSystemItem("events", {
    id,
    once: true,
    trigger: {
      kind: "Proximity",
      position: [snap(world.x), world.y ?? defaultPlacementY(), snap(world.z)],
      radius: 3,
    },
    actions: [{ kind: "SetFlag", flag_id: `${id}_done` }],
  });
  updateViewportHint(`Added event trigger ${id}.`);
}

function addDialogue() {
  pushSystemItem("dialogues", {
    id: uniqueSystemId("dialogues", "dialogue"),
    speaker: "Unknown",
    lines: ["New dialogue line."],
  });
}

function pushSystemItem(key, item) {
  state.level[key] = state.level[key] || [];
  state.level[key].push(item);
  markDirty();
  renderSystemPanels();
  updateViewportHint(`Added ${key.replaceAll("_", " ")} item.`);
}

function duplicateSystemItem(key, index) {
  const item = state.level[key]?.[index];
  if (!item) return;
  const clone = JSON.parse(JSON.stringify(item));
  if (clone.id) clone.id = uniqueSystemId(key, clone.id);
  state.level[key].splice(index + 1, 0, clone);
  markDirty();
  renderSystemPanels();
}

function removeSystemItem(key, index) {
  state.level[key]?.splice(index, 1);
  markDirty();
  renderSystemPanels();
}

function setSystemValue(item, key, value, rerender = false) {
  item[key] = value;
  markDirty();
  if (rerender) renderSystemPanels();
  draw();
}

function setTriggerValue(eventItem, key, value) {
  eventItem.trigger = eventItem.trigger || {};
  eventItem.trigger[key] = value;
  markDirty();
  draw();
}

function jsonRow(label, value, onChange) {
  const area = document.createElement("textarea");
  area.className = "compact-json";
  area.value = JSON.stringify(value || [], null, 2);
  area.addEventListener("change", () => {
    try {
      const parsed = JSON.parse(area.value || "[]");
      if (!Array.isArray(parsed)) throw new Error("Expected array");
      area.style.borderColor = "";
      onChange(parsed);
    } catch (_) {
      area.style.borderColor = "var(--danger)";
    }
  });
  return row(label, area);
}

function linesRow(label, lines, onChange) {
  const area = document.createElement("textarea");
  area.className = "compact-json";
  area.value = (lines || []).join("\n");
  area.addEventListener("change", () => onChange(area.value.split(/\r?\n/).map((line) => line.trim()).filter(Boolean)));
  return row(label, area);
}

function uniqueSystemId(key, prefix) {
  const used = new Set((state.level[key] || []).map((item) => item.id).filter(Boolean));
  const base = sanitizeAuthoringId(prefix || key) || "item";
  let id = base;
  let index = 1;
  while (used.has(id)) {
    id = `${base}_${index}`;
    index += 1;
  }
  return id;
}

function sanitizeAuthoringId(value) {
  return String(value ?? "")
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

function csvList(value) {
  return String(value ?? "")
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function textRow(label, value, onChange) {
  const input = document.createElement("input");
  input.className = "field";
  input.value = value;
  input.addEventListener("input", () => onChange(input.value));
  return row(label, input);
}

function numberRow(label, value, onChange) {
  const input = document.createElement("input");
  input.className = "field";
  input.type = "number";
  input.step = "0.1";
  input.value = Number(value || 0);
  input.addEventListener("input", () => onChange(numberValue(input.value)));
  return row(label, input);
}

function checkRow(label, value, onChange) {
  const input = document.createElement("input");
  input.type = "checkbox";
  input.checked = value;
  input.addEventListener("change", () => onChange(input.checked));
  return row(label, input);
}

function selectRow(label, value, options, onChange) {
  const select = document.createElement("select");
  for (const option of options) {
    const node = document.createElement("option");
    node.value = option;
    node.textContent = option || "None";
    select.append(node);
  }
  if (!options.includes(value)) {
    const node = document.createElement("option");
    node.value = value;
    node.textContent = value;
    select.append(node);
  }
  select.value = value;
  select.addEventListener("change", () => onChange(select.value));
  return row(label, select);
}

function vectorRow(label, value, onChange) {
  const wrap = document.createElement("div");
  wrap.className = "triple";
  const current = vector(value);
  const inputs = [];
  current.forEach((number, index) => {
    const input = document.createElement("input");
    input.className = "field";
    input.type = "number";
    input.step = "0.1";
    input.value = number;
    input.addEventListener("input", () => {
      const next = inputs.map((node, axis) => {
        const parsed = Number(node.value);
        return Number.isFinite(parsed) ? parsed : current[axis];
      });
      onChange(next);
    });
    inputs.push(input);
    wrap.append(input);
  });
  return row(label, wrap);
}

function row(label, control) {
  const wrap = document.createElement("div");
  wrap.className = "form-row";
  const labelNode = document.createElement("label");
  labelNode.textContent = label;
  wrap.append(labelNode, control);
  return wrap;
}

function setLevelValue(key, value) {
  state.level[key] = value;
  markDirty();
  renderObjects();
  draw();
}

function setPropValue(prop, key, value) {
  prop[key] = value;
  markDirty();
  renderObjects();
  draw();
}

function snapshotLevel() {
  return state.level ? JSON.stringify(state.level) : null;
}

function resetHistory(savedSnapshot = state.dirty ? null : snapshotLevel()) {
  state.history.past = [];
  state.history.future = [];
  state.history.current = snapshotLevel();
  state.history.saved = savedSnapshot;
  state.history.applying = false;
  state.history.transactionDepth = 0;
  state.history.transactionChanged = false;
  renderHistoryControls();
}

function beginHistoryTransaction() {
  if (state.history.transactionDepth === 0) {
    state.history.transactionChanged = false;
  }
  state.history.transactionDepth += 1;
}

function endHistoryTransaction() {
  if (state.history.transactionDepth === 0) {
    return;
  }
  state.history.transactionDepth -= 1;
  if (state.history.transactionDepth === 0 && state.history.transactionChanged) {
    state.history.transactionChanged = false;
    recordHistory();
  }
}

function recordHistory() {
  if (state.history.applying || !state.level) {
    return;
  }
  const snapshot = snapshotLevel();
  if (snapshot == null || snapshot === state.history.current) {
    return;
  }
  if (state.history.current != null) {
    state.history.past.push(state.history.current);
    if (state.history.past.length > state.history.limit) {
      state.history.past.shift();
    }
  }
  state.history.current = snapshot;
  state.history.future = [];
  renderHistoryControls();
}

function restoreHistorySnapshot(snapshot) {
  if (!snapshot) {
    return;
  }
  state.history.applying = true;
  try {
    state.level = normalizeLevel(JSON.parse(snapshot));
    clampSelections();
    state.dirty = state.history.saved == null || snapshot !== state.history.saved;
    queueLocalDraft();
    markValidationStale();
    renderAll();
    draw();
  } finally {
    state.history.applying = false;
  }
}

function undoHistory() {
  if (state.history.past.length === 0) {
    updateViewportHint("Nothing to undo.");
    return;
  }
  const current = state.history.current || snapshotLevel();
  if (current) {
    state.history.future.push(current);
  }
  const previous = state.history.past.pop();
  state.history.current = previous;
  restoreHistorySnapshot(previous);
  updateViewportHint("Undo.");
}

function redoHistory() {
  if (state.history.future.length === 0) {
    updateViewportHint("Nothing to redo.");
    return;
  }
  const current = state.history.current || snapshotLevel();
  if (current) {
    state.history.past.push(current);
  }
  const next = state.history.future.pop();
  state.history.current = next;
  restoreHistorySnapshot(next);
  updateViewportHint("Redo.");
}

function renderHistoryControls() {
  if (el.undoAction) {
    el.undoAction.disabled = state.history.past.length === 0;
  }
  if (el.redoAction) {
    el.redoAction.disabled = state.history.future.length === 0;
  }
}

function clampSelections() {
  const valid = selectionIndices();
  setSelection(valid, valid.includes(state.selectedProp) ? state.selectedProp : valid.at(-1));
}

function markDirty() {
  if (state.history.transactionDepth > 0) {
    state.history.transactionChanged = true;
  } else {
    recordHistory();
  }
  state.dirty = state.history.saved == null || snapshotLevel() !== state.history.saved;
  queueLocalDraft();
  markValidationStale();
  renderProject();
  renderHistoryControls();
}

function markValidationStale() {
  state.validation = { current: false, ok: false, errors: [] };
  renderValidation();
}

function renderValidation() {
  if (!state.validation.current) {
    el.validationBadge.className = "badge warn";
    el.validationBadge.textContent = "Check needed";
    el.validationList.textContent = "Press Validate before saving or testing.";
    return;
  }
  if (state.validation.ok) {
    setValidationOk("Level validation passed.");
    return;
  }
  el.validationBadge.className = "badge bad";
  el.validationBadge.textContent = `${state.validation.errors.length} issues`;
  el.validationList.innerHTML = "";
  for (const error of state.validation.errors) {
    const target = validationPropIndex(error);
    const node = document.createElement(target == null ? "div" : "button");
    node.className = "issue";
    node.textContent = error;
    if (target != null) {
      node.type = "button";
      node.title = "Select this prop";
      node.addEventListener("click", () => {
        setSelection([target], target);
        setWorkspaceTab("props");
        renderAll();
        focusSelected();
      });
    }
    el.validationList.append(node);
  }
}

function validationPropIndex(message) {
  const match = String(message).match(/\bprop\s+(\d+)\b/i);
  if (!match) {
    return null;
  }
  const index = Number(match[1]);
  return state.level?.props?.[index] ? index : null;
}

function setValidationOk(message) {
  el.validationBadge.className = "badge ok";
  el.validationBadge.textContent = "OK";
  el.validationList.innerHTML = `<div class="ok">${escapeHtml(message)}</div>`;
}

function setProjectStatus(message) {
  el.projectSummary.textContent = message;
}

function addPropAt(world) {
  if (!state.level) {
    updateViewportHint("Load a level before placing props.");
    return null;
  }
  if (!state.selectedTemplate) {
    updateViewportHint("Choose an item from the placement palette first.");
    return;
  }
  state.lastPlacementLabel = null;
  if (state.selectedTemplate.group === "prefab") {
    return addPrefabAt(world, state.selectedTemplate);
  }
  const y = world.y ?? defaultPlacementY();
  const prop = {
    ...propDefaults([snap(world.x), y, snap(world.z)]),
    ...templateToProp(state.selectedTemplate),
  };
  prop.position = [snap(world.x), y, snap(world.z)];
  if (prop.anchor_id === "anchor") {
    prop.anchor_id = `anchor_${state.level.props.length + 1}`;
  }
  state.level.props.push(prop);
  setSelection([state.level.props.length - 1], state.level.props.length - 1);
  state.lastPlacementLabel = propLabel(prop, state.level.props.length - 1);
  markDirty();
  renderAll();
  return prop;
}

function addPrefabAt(world, template) {
  const prefab = template.prefab;
  if (!window.CenotaphPrefabTools || !prefab) {
    updateViewportHint("The selected prefab could not be loaded. Select it again in Prefabs.");
    return null;
  }
  const y = Number.isFinite(Number(world.y)) ? Number(world.y) : defaultPlacementY();
  let clones;
  try {
    clones = window.CenotaphPrefabTools.instantiate(prefab, [snap(world.x), y, snap(world.z)]);
  } catch (error) {
    updateViewportHint(`Prefab placement failed: ${error.message}`);
    return null;
  }

  const prefabId = template.prefab_id || slugFromPath(prefab.name || "prefab");
  const usedAnchors = new Set(
    state.level.props.map((prop) => prop.anchor_id).filter(Boolean)
  );
  const created = [];
  for (const source of clones) {
    const prop = {
      ...propDefaults(),
      ...source,
      position: vector(source.position),
      rotation: vector(source.rotation),
      scale: vector(source.scale, [1, 1, 1]),
    };
    prop.id = uniquePlacedPropId(`${prefabId}_${source.id || slugFromPath(source.asset_id || "prop")}`);
    if (source.anchor_id) {
      prop.anchor_id = uniqueTextId(`${prefabId}_${source.anchor_id}`, usedAnchors);
    }
    if (prop.brush_geometry) {
      normalizeBrushGeometry(prop.brush_geometry);
    }
    state.level.props.push(prop);
    created.push(state.level.props.length - 1);
  }
  setSelection(created, created.at(-1));
  state.lastPlacementLabel = `${prefab.name || prefabId} (${created.length} ${created.length === 1 ? "prop" : "props"})`;
  markDirty();
  renderAll();
  return state.level.props[created.at(-1)] || null;
}

function placementLabel(prop) {
  return state.lastPlacementLabel || propLabel(prop, state.selectedProp);
}

function createDrawGeometry() {
  const start = state.drawBrush.start;
  const current = state.drawBrush.current;
  if (!state.level) {
    updateViewportHint("Load a level before drawing geometry.");
    return null;
  }
  if (!start || !current) {
    return null;
  }
  const prop = drawBrushProp(start, current);
  if (!prop) {
    updateViewportHint("Brush was not created because its footprint is too small. Drag farther and try again.");
    return null;
  }
  state.level.props.push(prop);
  setSelection([state.level.props.length - 1], state.level.props.length - 1);
  markDirty();
  renderAll();
  return prop;
}

function normalizeDrawView(viewName = state.drawBrush.viewName) {
  return viewName === "front" || viewName === "side" ? viewName : "top";
}

function pointArray(point) {
  return [numberValue(point?.x), numberValue(point?.y), numberValue(point?.z)];
}

function snappedPoint(point) {
  const values = pointArray(point).map((value) => snap(value));
  return { x: values[0], y: values[1], z: values[2] };
}

function drawBrushProp(start, current, viewName = state.drawBrush.viewName) {
  const brush = currentBrush();
  const dimensions = drawBrushDimensions(start, current, brush, viewName);
  let geometry;
  if (brush.kind === "slope") {
    geometry = slopeBrushGeometry(dimensions.scale, brush.direction);
  } else if (brush.kind === "cylinder") {
    geometry = cylinderBrushGeometry(dimensions.scale, brush.segments);
  } else if (brush.kind === "stairs") {
    geometry = stairBrushGeometry(dimensions.scale, brush.steps, brush.direction);
  } else if (brush.kind === "terrain") {
    geometry = terrainBrushGeometry(
      dimensions.scale,
      brush.terrainResolution,
      brush.terrainSeed,
      dimensions.terrainRelief,
      brush.thickness
    );
  } else {
    geometry = boxBrushGeometry(dimensions.scale);
  }
  return {
    ...propDefaults(dimensions.center),
    id: nextBrushId(),
    asset_id: dimensions.assetId,
    position: dimensions.center,
    scale: [1, 1, 1],
    collider_type: "Mesh",
    brush_geometry: geometry,
  };
}

function drawBrushDimensions(start, current, brush = currentBrush(), requestedView = state.drawBrush.viewName) {
  const viewName = normalizeDrawView(requestedView);
  const view = orthoView(viewName);
  const startPoint = pointArray(start);
  const currentPoint = pointArray(current);
  const minimumExtent = Math.max(0.1, state.gridSize);
  const scale = [Math.max(0.1, brush.height), Math.max(0.1, brush.height), Math.max(0.1, brush.height)];
  const center = [0, 0, 0];
  const minH = Math.min(startPoint[view.h], currentPoint[view.h]);
  const maxH = Math.max(startPoint[view.h], currentPoint[view.h]);
  const minV = Math.min(startPoint[view.v], currentPoint[view.v]);
  const maxV = Math.max(startPoint[view.v], currentPoint[view.v]);
  scale[view.h] = Math.max(minimumExtent, Math.abs(snap(maxH - minH)));
  scale[view.v] = Math.max(minimumExtent, Math.abs(snap(maxV - minV)));
  center[view.h] = snap((minH + maxH) * 0.5);
  center[view.v] = snap((minV + maxV) * 0.5);

  const thickness = Math.max(0.1, brush.thickness);
  if (viewName === "top" && brush.kind === "floor") {
    scale[1] = thickness;
  } else if (brush.kind === "wall") {
    if (viewName === "top") {
      if (scale[0] >= scale[2]) {
        scale[2] = thickness;
      } else {
        scale[0] = thickness;
      }
    } else {
      scale[view.plane] = thickness;
    }
  }

  let terrainRelief = Math.max(0, brush.terrainRelief);
  if (brush.kind === "terrain") {
    if (viewName === "top") {
      scale[1] = Math.max(thickness + terrainRelief, thickness + 0.1);
    } else {
      scale[1] = Math.max(scale[1], thickness + 0.1);
      terrainRelief = Math.min(terrainRelief, Math.max(0, scale[1] - thickness));
    }
  }

  const planeCoordinate = viewName === "top"
    ? brush.groundY
    : viewName === "front"
      ? brush.frontZ
      : brush.sideX;
  center[view.plane] = viewName === "top"
    ? planeCoordinate + scale[view.plane] * 0.5
    : planeCoordinate;

  return {
    center,
    scale,
    terrainRelief,
    viewName,
    assetId: ["slope", "cylinder", "stairs", "terrain"].includes(brush.kind)
      ? "editor/brush_geometry"
      : brush.kind === "floor"
        ? "props/test_platform.obj"
        : "props/test_wall.obj",
  };
}

function currentBrush() {
  state.drawBrush.kind = el.brushKind.value;
  state.drawBrush.height = readNumberInput(el.brushHeight, 3, 0.1);
  state.drawBrush.thickness = readNumberInput(el.brushThickness, 0.5, 0.1);
  state.drawBrush.direction = el.brushDirection.value;
  state.drawBrush.segments = Math.round(clamp(readNumberInput(el.brushSegments, 12, 3), 3, 64));
  state.drawBrush.steps = Math.round(clamp(readNumberInput(el.brushSteps, 6, 2), 2, 32));
  state.drawBrush.terrainResolution = Math.round(clamp(readNumberInput(el.terrainResolution, 8, 2), 2, 24));
  state.drawBrush.terrainRelief = clamp(readNumberInput(el.terrainRelief, 3, 0), 0, 256);
  state.drawBrush.terrainSeed = Math.round(readNumberInput(el.terrainSeed, 1));
  state.drawBrush.groundY = readNumberInput(el.brushGround, defaultPlacementY());
  state.drawBrush.frontZ = readNumberInput(el.brushFrontZ, 0);
  state.drawBrush.sideX = readNumberInput(el.brushSideX, 0);
  return state.drawBrush;
}

function convertPropToBrush(prop, preset = "box", direction = "x+") {
  const size = editableBrushSize(prop);
  if (preset === "slope") {
    prop.brush_geometry = slopeBrushGeometry(size, direction);
  } else if (preset === "cylinder") {
    prop.brush_geometry = cylinderBrushGeometry(size, state.drawBrush.segments);
  } else if (preset === "stairs") {
    prop.brush_geometry = stairBrushGeometry(size, state.drawBrush.steps, direction);
  } else if (preset === "terrain") {
    const terrainHeight = Math.max(
      size[1],
      state.drawBrush.thickness + state.drawBrush.terrainRelief
    );
    prop.brush_geometry = terrainBrushGeometry(
      [size[0], terrainHeight, size[2]],
      state.drawBrush.terrainResolution,
      state.drawBrush.terrainSeed,
      state.drawBrush.terrainRelief,
      state.drawBrush.thickness
    );
  } else {
    prop.brush_geometry = boxBrushGeometry(size);
  }
  prop.scale = [1, 1, 1];
  prop.collider_type = "Mesh";
  if (!prop.asset_id || preset !== "box") {
    prop.asset_id = "editor/brush_geometry";
  }
  markDirty();
  renderAll();
  updateViewportHint(`Converted ${propLabel(prop, state.selectedProp ?? 0)} to ${preset} geometry.`);
}

function removeBrushGeometry(prop) {
  delete prop.brush_geometry;
  if (prop.asset_id === "editor/brush_geometry") {
    prop.asset_id = "props/test_wall.obj";
  }
  if (prop.collider_type === "Mesh") {
    prop.collider_type = "Box";
  }
  markDirty();
  renderAll();
  updateViewportHint("Custom brush geometry removed.");
}

function flattenBrushTop(prop) {
  const geometry = normalizeBrushGeometry(prop.brush_geometry);
  const topY = Math.max(...geometry.vertices.map((vertex) => vertex[1]));
  for (const vertex of geometry.vertices) {
    if (vertex[1] > 0) {
      vertex[1] = topY;
    }
  }
  markDirty();
  renderInspector();
  draw();
  updateViewportHint("Top vertices flattened.");
}

function mirrorBrush(prop, axis) {
  const geometry = normalizeBrushGeometry(prop.brush_geometry);
  for (const vertex of geometry.vertices) {
    vertex[axis] *= -1;
  }
  geometry.faces = geometry.faces.map(([a, b, c]) => [a, c, b]);
  markDirty();
  renderInspector();
  draw();
  updateViewportHint(axis === 0 ? "Brush mirrored on X." : "Brush mirrored on Z.");
}

function snapBrushVertices(prop) {
  const geometry = normalizeBrushGeometry(prop.brush_geometry);
  for (const vertex of geometry.vertices) {
    vertex[0] = gridSnap(vertex[0]);
    vertex[1] = gridSnap(vertex[1]);
    vertex[2] = gridSnap(vertex[2]);
  }
  markDirty();
  renderInspector();
  draw();
  updateViewportHint("Brush vertices snapped to grid.");
}

function recenterBrushGeometry(prop) {
  const geometry = normalizeBrushGeometry(prop.brush_geometry);
  const bounds = localBrushBounds(geometry);
  const center = scale3(add3(bounds.min, bounds.max), 0.5);
  const worldOffset = transformLocalOffset(prop, center);
  prop.position = add3(vector(prop.position), worldOffset);
  for (const vertex of geometry.vertices) {
    vertex[0] -= center[0];
    vertex[1] -= center[1];
    vertex[2] -= center[2];
  }
  markDirty();
  renderInspector();
  renderObjects();
  draw();
  updateViewportHint("Brush origin recentered without moving the visible geometry.");
}

function editableBrushSize(prop) {
  if (prop.brush_geometry) {
    const bounds = localBrushBounds(normalizeBrushGeometry(prop.brush_geometry));
    return sub3(bounds.max, bounds.min).map((value) => Math.max(0.2, Math.abs(value)));
  }
  return vector(prop.scale).map((value) => Math.max(0.2, Math.abs(value)));
}

function boxBrushGeometry(scale) {
  const half = vector(scale).map((value) => Math.max(0.05, Math.abs(value) * 0.5));
  const vertices = [];
  for (const x of [-half[0], half[0]]) {
    for (const y of [-half[1], half[1]]) {
      for (const z of [-half[2], half[2]]) {
        vertices.push([x, y, z]);
      }
    }
  }
  return { vertices, faces: brushFaces.map((face) => [...face]) };
}

function slopeBrushGeometry(scale, direction = "x+") {
  const geometry = boxBrushGeometry(scale);
  const axis = direction.startsWith("z") ? 2 : 0;
  const highSign = direction.endsWith("+") ? 1 : -1;
  const yValues = geometry.vertices.map((vertex) => vertex[1]);
  const lowY = Math.min(...yValues);
  const highY = Math.max(...yValues);
  const lip = Math.min(0.25, Math.max(0.05, Math.abs(highY - lowY) * 0.15));
  for (const vertex of geometry.vertices) {
    if (vertex[1] <= 0) {
      continue;
    }
    vertex[1] = Math.sign(vertex[axis]) === highSign ? highY : lowY + lip;
  }
  return geometry;
}

function cylinderBrushGeometry(scale, segmentCount = 12) {
  const size = vector(scale).map((value) => Math.max(0.1, Math.abs(value)));
  const segments = Math.round(clamp(segmentCount, 3, 64));
  const halfWidth = size[0] * 0.5;
  const halfHeight = size[1] * 0.5;
  const halfDepth = size[2] * 0.5;
  const vertices = [];
  const bottom = [];
  const top = [];
  for (let index = 0; index < segments; index += 1) {
    const angle = (index / segments) * Math.PI * 2;
    bottom.push(vertices.length);
    vertices.push([Math.cos(angle) * halfWidth, -halfHeight, Math.sin(angle) * halfDepth]);
    top.push(vertices.length);
    vertices.push([Math.cos(angle) * halfWidth, halfHeight, Math.sin(angle) * halfDepth]);
  }

  const faces = [];
  for (let index = 0; index < segments; index += 1) {
    const next = (index + 1) % segments;
    faces.push([bottom[index], bottom[next], top[next]], [bottom[index], top[next], top[index]]);
  }
  for (let index = 1; index < segments - 1; index += 1) {
    faces.push([bottom[0], bottom[index + 1], bottom[index]]);
    faces.push([top[0], top[index], top[index + 1]]);
  }
  return { vertices, faces };
}

function stairBrushGeometry(scale, stepCount = 6, direction = "x+") {
  const size = vector(scale).map((value) => Math.max(0.1, Math.abs(value)));
  const steps = Math.round(clamp(stepCount, 2, 32));
  const halfWidth = size[0] * 0.5;
  const halfHeight = size[1] * 0.5;
  const halfDepth = size[2] * 0.5;
  const stepWidth = size[0] / steps;
  const stepHeight = size[1] / steps;
  const vertices = [];
  const faces = [];
  const addQuad = (a, b, c, d) => {
    const base = vertices.length;
    vertices.push(a, b, c, d);
    faces.push([base, base + 1, base + 2], [base, base + 2, base + 3]);
  };

  for (let index = 0; index < steps; index += 1) {
    const x0 = -halfWidth + index * stepWidth;
    const x1 = x0 + stepWidth;
    const previousTop = -halfHeight + index * stepHeight;
    const topY = previousTop + stepHeight;
    addQuad(
      [x0, topY, -halfDepth],
      [x0, topY, halfDepth],
      [x1, topY, halfDepth],
      [x1, topY, -halfDepth]
    );
    addQuad(
      [x0, previousTop, -halfDepth],
      [x0, previousTop, halfDepth],
      [x0, topY, halfDepth],
      [x0, topY, -halfDepth]
    );
    addQuad(
      [x0, -halfHeight, -halfDepth],
      [x0, topY, -halfDepth],
      [x1, topY, -halfDepth],
      [x1, -halfHeight, -halfDepth]
    );
    addQuad(
      [x0, -halfHeight, halfDepth],
      [x1, -halfHeight, halfDepth],
      [x1, topY, halfDepth],
      [x0, topY, halfDepth]
    );
  }
  addQuad(
    [-halfWidth, -halfHeight, -halfDepth],
    [halfWidth, -halfHeight, -halfDepth],
    [halfWidth, -halfHeight, halfDepth],
    [-halfWidth, -halfHeight, halfDepth]
  );
  addQuad(
    [halfWidth, -halfHeight, -halfDepth],
    [halfWidth, halfHeight, -halfDepth],
    [halfWidth, halfHeight, halfDepth],
    [halfWidth, -halfHeight, halfDepth]
  );

  for (const vertex of vertices) {
    const [x, y, z] = vertex;
    if (direction === "x-") {
      vertex.splice(0, 3, -x, y, -z);
    } else if (direction === "z+") {
      vertex.splice(0, 3, -z, y, x);
    } else if (direction === "z-") {
      vertex.splice(0, 3, z, y, -x);
    }
  }
  return { vertices, faces };
}

function terrainBrushGeometry(scale, resolution = 8, seed = 1, relief = 3, baseThickness = 0.5) {
  const size = vector(scale).map((value) => Math.max(0.1, Math.abs(value)));
  const columns = Math.round(clamp(resolution, 2, 24));
  const rows = columns;
  const terrainSeed = Math.round(numberValue(seed));
  const bottomY = -size[1] * 0.5;
  const thickness = clamp(baseThickness, 0.1, Math.max(0.1, size[1] - 0.05));
  const appliedRelief = clamp(relief, 0, Math.max(0, size[1] - thickness));
  const baseY = bottomY + thickness;
  const vertices = [];

  for (let rowIndex = 0; rowIndex <= rows; rowIndex += 1) {
    const v = rowIndex / rows;
    for (let columnIndex = 0; columnIndex <= columns; columnIndex += 1) {
      const u = columnIndex / columns;
      const x = -size[0] * 0.5 + u * size[0];
      const z = -size[2] * 0.5 + v * size[2];
      const height = terrainHeightSample(u, v, terrainSeed) * appliedRelief;
      vertices.push([x, baseY + height, z]);
    }
  }

  const topVertexCount = vertices.length;
  for (let rowIndex = 0; rowIndex <= rows; rowIndex += 1) {
    const v = rowIndex / rows;
    for (let columnIndex = 0; columnIndex <= columns; columnIndex += 1) {
      const u = columnIndex / columns;
      vertices.push([
        -size[0] * 0.5 + u * size[0],
        bottomY,
        -size[2] * 0.5 + v * size[2],
      ]);
    }
  }

  const indexAt = (columnIndex, rowIndex) => rowIndex * (columns + 1) + columnIndex;
  const faces = [];
  for (let rowIndex = 0; rowIndex < rows; rowIndex += 1) {
    for (let columnIndex = 0; columnIndex < columns; columnIndex += 1) {
      const a = indexAt(columnIndex, rowIndex);
      const b = indexAt(columnIndex + 1, rowIndex);
      const c = indexAt(columnIndex, rowIndex + 1);
      const d = indexAt(columnIndex + 1, rowIndex + 1);
      faces.push([a, c, b], [b, c, d]);

      const bottomA = topVertexCount + a;
      const bottomB = topVertexCount + b;
      const bottomC = topVertexCount + c;
      const bottomD = topVertexCount + d;
      faces.push([bottomA, bottomB, bottomC], [bottomB, bottomD, bottomC]);
    }
  }

  const addBoundaryQuad = (topA, topB) => {
    const bottomA = topVertexCount + topA;
    const bottomB = topVertexCount + topB;
    faces.push([topA, topB, bottomB], [topA, bottomB, bottomA]);
  };
  for (let columnIndex = 0; columnIndex < columns; columnIndex += 1) {
    addBoundaryQuad(indexAt(columnIndex, 0), indexAt(columnIndex + 1, 0));
  }
  for (let rowIndex = 0; rowIndex < rows; rowIndex += 1) {
    addBoundaryQuad(indexAt(columns, rowIndex), indexAt(columns, rowIndex + 1));
  }
  for (let columnIndex = columns; columnIndex > 0; columnIndex -= 1) {
    addBoundaryQuad(indexAt(columnIndex, rows), indexAt(columnIndex - 1, rows));
  }
  for (let rowIndex = rows; rowIndex > 0; rowIndex -= 1) {
    addBoundaryQuad(indexAt(0, rowIndex), indexAt(0, rowIndex - 1));
  }

  return {
    kind: "terrain",
    terrain: {
      columns,
      rows,
      seed: terrainSeed,
      relief: appliedRelief,
      base_thickness: thickness,
      sculpt_strength: Math.max(0.1, appliedRelief * 0.2 || 0.5),
    },
    vertices,
    faces,
  };
}

function terrainHeightSample(u, v, seed) {
  const broad = terrainNoise(u * 3.2, v * 3.2, seed);
  const detail = terrainNoise(u * 8.4, v * 8.4, seed + 17);
  const wave = (Math.sin((u * 1.7 + v * 0.8 + seed * 0.013) * Math.PI * 2) + 1) * 0.5;
  const edgeDistance = Math.min(u, v, 1 - u, 1 - v);
  const edgeBlend = smoothStep(0, 0.16, edgeDistance);
  return clamp((broad * 0.58 + detail * 0.24 + wave * 0.18) * (0.45 + edgeBlend * 0.55), 0, 1);
}

function terrainNoise(x, z, seed) {
  const x0 = Math.floor(x);
  const z0 = Math.floor(z);
  const tx = x - x0;
  const tz = z - z0;
  const sx = tx * tx * (3 - 2 * tx);
  const sz = tz * tz * (3 - 2 * tz);
  const a = terrainHash(x0, z0, seed);
  const b = terrainHash(x0 + 1, z0, seed);
  const c = terrainHash(x0, z0 + 1, seed);
  const d = terrainHash(x0 + 1, z0 + 1, seed);
  return lerp(lerp(a, b, sx), lerp(c, d, sx), sz);
}

function terrainHash(x, z, seed) {
  const value = Math.sin(x * 127.1 + z * 311.7 + seed * 74.7) * 43758.5453123;
  return value - Math.floor(value);
}

function normalizeBrushGeometry(geometry) {
  if (!geometry) {
    return null;
  }
  geometry.vertices = (geometry.vertices || [])
    .map((vertex) => vector(vertex).map((value) => Number.isFinite(value) ? value : 0))
    .filter((vertex) => vertex.every(Number.isFinite));
  geometry.faces = (geometry.faces || [])
    .map((face) => Array.isArray(face) ? face.slice(0, 3).map((value) => Math.floor(Number(value))) : null)
    .filter((face) =>
      face &&
      face.length === 3 &&
      face.every((index) => Number.isInteger(index) && index >= 0 && index < geometry.vertices.length) &&
      new Set(face).size === 3
    );
  if (geometry.faces.length === 0 && geometry.vertices.length >= 8) {
    geometry.faces = brushFaces.map((face) => [...face]);
  }
  return geometry;
}

function localBrushBounds(geometry) {
  const vertices = normalizeBrushGeometry(geometry)?.vertices || [];
  if (vertices.length === 0) {
    return { min: [-0.5, -0.5, -0.5], max: [0.5, 0.5, 0.5] };
  }
  const min = [...vertices[0]];
  const max = [...vertices[0]];
  for (const vertex of vertices.slice(1)) {
    for (let axis = 0; axis < 3; axis += 1) {
      min[axis] = Math.min(min[axis], vertex[axis]);
      max[axis] = Math.max(max[axis], vertex[axis]);
    }
  }
  return { min, max };
}

function transformLocalOffset(prop, local) {
  const scale = vector(prop.scale);
  const scaled = [local[0] * scale[0], local[1] * scale[1], local[2] * scale[2]];
  const yaw = ((prop.rotation?.[1] || 0) * Math.PI) / 180;
  const cos = Math.cos(yaw);
  const sin = Math.sin(yaw);
  return [
    scaled[0] * cos - scaled[2] * sin,
    scaled[1],
    scaled[0] * sin + scaled[2] * cos,
  ];
}

function transformedBrushVertices(prop) {
  const geometry = normalizeBrushGeometry(prop.brush_geometry);
  if (!geometry) {
    return [];
  }
  const position = vector(prop.position);
  return geometry.vertices.map((vertex) => add3(position, transformLocalOffset(prop, vertex)));
}

function propVertices(prop) {
  if (prop.brush_geometry) {
    const vertices = transformedBrushVertices(prop);
    if (vertices.length > 0) {
      return vertices;
    }
  }
  const center = vector(prop.position);
  const scale = vector(prop.scale).map((value) => Math.max(0.2, Math.abs(value)));
  const yaw = ((prop.rotation?.[1] || 0) * Math.PI) / 180;
  return cubeCorners(center, scale, yaw);
}

function propBounds(prop) {
  const vertices = propVertices(prop);
  const min = [...vertices[0]];
  const max = [...vertices[0]];
  for (const vertex of vertices.slice(1)) {
    for (let axis = 0; axis < 3; axis += 1) {
      min[axis] = Math.min(min[axis], vertex[axis]);
      max[axis] = Math.max(max[axis], vertex[axis]);
    }
  }
  return { min, max, center: scale3(add3(min, max), 0.5), size: sub3(max, min) };
}

function brushTriangles(prop) {
  const geometry = normalizeBrushGeometry(prop.brush_geometry);
  if (!geometry) {
    return [];
  }
  const vertices = transformedBrushVertices(prop);
  return geometry.faces.flatMap((face) => face.flatMap((index) => vertices[index] || []));
}

function brushLines(prop) {
  const geometry = normalizeBrushGeometry(prop.brush_geometry);
  if (!geometry) {
    return [];
  }
  const vertices = transformedBrushVertices(prop);
  const edges = new Set();
  const lines = [];
  for (const face of geometry.faces) {
    for (const [a, b] of [[face[0], face[1]], [face[1], face[2]], [face[2], face[0]]]) {
      const key = a < b ? `${a}:${b}` : `${b}:${a}`;
      if (edges.has(key) || !vertices[a] || !vertices[b]) {
        continue;
      }
      edges.add(key);
      lines.push(...vertices[a], ...vertices[b]);
    }
  }
  return lines;
}

function nextBrushId() {
  const used = new Set((state.level?.props || []).map((prop) => prop.id).filter(Boolean));
  let index = (state.level?.props || []).length + 1;
  let id = `brush_${index}`;
  while (used.has(id)) {
    index += 1;
    id = `brush_${index}`;
  }
  return id;
}

function readNumberInput(input, fallback, min = null) {
  const parsed = Number(input.value);
  const value = Number.isFinite(parsed) ? parsed : fallback;
  return min == null ? value : Math.max(min, value);
}

function templateToProp(template) {
  const prop = { ...template };
  delete prop.group;
  delete prop.label;
  return prop;
}

function deleteSelected() {
  const indices = selectionIndices();
  if (!state.level || indices.length === 0) {
    return;
  }
  for (const index of [...indices].sort((left, right) => right - left)) {
    state.level.props.splice(index, 1);
  }
  resetSelection();
  markDirty();
  renderAll();
  updateViewportHint(`Deleted ${indices.length} ${indices.length === 1 ? "prop" : "props"}.`);
}

function duplicateSelected() {
  const indices = selectionIndices();
  if (!state.level || indices.length === 0) {
    updateViewportHint("Select props before duplicating.");
    return null;
  }
  const created = [];
  for (const index of indices) {
    const source = state.level.props[index];
    const clone = cloneProp(source);
    clone.position = add3(vector(source.position), [2, 0, 2]);
    clone.id = uniquePropId(source.id || slugFromPath(source.asset_id || "prop"));
    state.level.props.push(clone);
    created.push(state.level.props.length - 1);
  }
  setSelection(created, created.at(-1));
  markDirty();
  renderAll();
  updateViewportHint(`Duplicated ${created.length} ${created.length === 1 ? "prop" : "props"}.`);
  return created.map((index) => state.level.props[index]);
}

function copySelected() {
  const indices = selectionIndices();
  if (!state.level || indices.length === 0) {
    updateViewportHint("Select props before copying.");
    return;
  }
  state.clipboardProps = indices.map((index) => cloneProp(state.level.props[index]));
  state.clipboardCenter = selectionCenter(indices);
  updateViewportHint(`Copied ${indices.length} ${indices.length === 1 ? "prop" : "props"}.`);
}

function moveSelectedTo(world) {
  const indices = selectionIndices();
  if (!state.level || indices.length === 0) {
    updateViewportHint("Select props before moving them.");
    return null;
  }
  if (!world) {
    updateViewportHint("No valid placement point under the cursor.");
    return null;
  }
  const center = selectionCenter(indices);
  moveSelectionBy([world.x - center[0], 0, world.z - center[2]]);
  renderAll();
  updateViewportHint(`Moved ${indices.length} ${indices.length === 1 ? "prop" : "props"}.`);
  return indices.map((index) => state.level.props[index]);
}

function pasteCopied(world = null) {
  if (!state.level || state.clipboardProps.length === 0) {
    updateViewportHint("Copy props before pasting.");
    return null;
  }
  const sourceCenter = state.clipboardCenter || [0, 0, 0];
  const offset = world
    ? [world.x - sourceCenter[0], 0, world.z - sourceCenter[2]]
    : [2, 0, 2];
  const created = [];
  for (const copied of state.clipboardProps) {
    const clone = cloneProp(copied);
    clone.position = add3(vector(clone.position), offset).map((value, axis) =>
      axis === 1 ? value : snap(value)
    );
    clone.id = uniquePropId(clone.id || slugFromPath(clone.asset_id || "prop"));
    state.level.props.push(clone);
    created.push(state.level.props.length - 1);
  }
  setSelection(created, created.at(-1));
  markDirty();
  renderAll();
  updateViewportHint(`Pasted ${created.length} ${created.length === 1 ? "prop" : "props"}.`);
  return created.map((index) => state.level.props[index]);
}

function clearSelection() {
  if (selectionIndices().length === 0) {
    return;
  }
  resetSelection();
  renderAll();
  updateViewportHint("Selection cleared.");
}

function focusSelected() {
  const indices = selectionIndices();
  if (!state.level || indices.length === 0) {
    updateViewportHint("Select props before focusing.");
    return;
  }
  const bounds = selectionBounds(indices);
  const center = bounds.center;
  const scale = bounds.size.map((value) => Math.max(1, Math.abs(value)));
  const distance = Math.max(10, Math.max(scale[0], scale[1], scale[2]) * 4);
  state.camera.position = [center[0], center[1] + Math.max(2, scale[1]), center[2] + distance];
  state.camera.yaw = 0;
  state.camera.pitch = -0.18;
  state.ortho.center = [...center];
  updateViewportHint(`Focused ${indices.length} ${indices.length === 1 ? "prop" : "props"}.`);
  draw();
}

function selectionBounds(indices = selectionIndices()) {
  const first = propBounds(state.level.props[indices[0]]);
  const min = [...first.min];
  const max = [...first.max];
  for (const index of indices.slice(1)) {
    const bounds = propBounds(state.level.props[index]);
    for (let axis = 0; axis < 3; axis += 1) {
      min[axis] = Math.min(min[axis], bounds.min[axis]);
      max[axis] = Math.max(max[axis], bounds.max[axis]);
    }
  }
  return { min, max, center: scale3(add3(min, max), 0.5), size: sub3(max, min) };
}

function axisAllows(axis) {
  return state.transform.axis === "all" || state.transform.axis === ["x", "y", "z"][axis];
}

function moveSelectionBy(delta, options = {}) {
  const indices = selectionIndices();
  if (indices.length === 0) {
    return;
  }
  const constrained = vector(delta).map((value, axis) =>
    options.respectAxis === false || axisAllows(axis) ? value : 0
  );
  if (length3(constrained) <= 0.000001) {
    return;
  }
  for (const index of indices) {
    const prop = state.level.props[index];
    const current = vector(prop.position);
    prop.position = add3(current, constrained).map((value, axis) => {
      if (Math.abs(constrained[axis]) <= 0.000001) {
        return current[axis];
      }
      return options.snap === false ? value : snap(value);
    });
  }
  markDirty();
  renderObjects();
  renderSelectionSummary();
  draw();
}

function nudgeSelection(direction) {
  const amount = Math.max(0.1, Number(state.gridSize) || 1);
  moveSelectionBy(scale3(direction, amount), { respectAxis: false, snap: false });
  renderInspector();
  updateViewportHint(`Nudged selection by ${amount}.`);
}

function rotateSelection(axis, degrees) {
  const indices = selectionIndices();
  if (indices.length === 0) {
    return;
  }
  for (const index of indices) {
    const prop = state.level.props[index];
    const rotation = vector(prop.rotation);
    rotation[axis] += degrees;
    prop.rotation = rotation;
  }
  markDirty();
  renderInspector();
  renderObjects();
  draw();
  updateViewportHint(`Rotated ${indices.length} ${indices.length === 1 ? "prop" : "props"} by ${degrees} degrees.`);
}

function scaleSelection(factor) {
  const indices = selectionIndices();
  if (indices.length === 0) {
    return;
  }
  for (const index of indices) {
    const prop = state.level.props[index];
    prop.scale = vector(prop.scale).map((value, axis) =>
      axisAllows(axis) ? Math.max(0.05, Math.abs(value * factor)) * Math.sign(value || 1) : value
    );
  }
  markDirty();
  renderInspector();
  renderObjects();
  draw();
  updateViewportHint(`Scaled selection by ${factor}.`);
}

function resetSelectionRotation() {
  for (const index of selectionIndices()) {
    state.level.props[index].rotation = [0, 0, 0];
  }
  markDirty();
  renderAll();
  updateViewportHint("Selection rotation reset.");
}

function resetSelectionScale() {
  for (const index of selectionIndices()) {
    state.level.props[index].scale = [1, 1, 1];
  }
  markDirty();
  renderAll();
  updateViewportHint("Selection scale reset.");
}

function cloneProp(prop) {
  return JSON.parse(JSON.stringify(prop));
}

function uniquePropId(base) {
  const used = new Set((state.level?.props || []).map((prop) => prop.id).filter(Boolean));
  base = slugFromPath(base || "prop");
  let id = `${base}_copy`;
  let index = 2;
  while (used.has(id)) {
    id = `${base}_copy_${index}`;
    index += 1;
  }
  return id;
}

function uniquePlacedPropId(base) {
  const used = new Set((state.level?.props || []).map((prop) => prop.id).filter(Boolean));
  const root = slugFromPath(base || "prop");
  let id = root;
  let index = 2;
  while (used.has(id)) {
    id = `${root}_${index}`;
    index += 1;
  }
  return id;
}

function uniqueTextId(base, used) {
  const root = slugFromPath(base || "id");
  let id = root;
  let index = 2;
  while (used.has(id)) {
    id = `${root}_${index}`;
    index += 1;
  }
  used.add(id);
  return id;
}

function setTool(tool) {
  if (!state.level || state.levelLoading) {
    state.tool = "select";
    renderAvailabilityControls();
    updateViewportHint(
      state.levelLoading
        ? "Wait for the level to finish loading before using editor tools."
        : state.connection.ready
          ? "Choose or create a level before using editor tools."
          : "Reconnect the editor session before loading or editing levels."
    );
    draw();
    return;
  }
  state.tool = tool;
  for (const [name, button] of [
    ["select", el.toolSelect],
    ["move", el.toolMove],
    ["place", el.toolPlace],
    ["draw", el.toolDraw],
  ]) {
    button.classList.toggle("active", tool === name);
  }
  updateViewportHint();
  draw();
}

function setViewLayout(layout) {
  state.viewLayout = layout;
  el.viewportGrid.classList.toggle("quad", layout === "quad");
  el.viewportGrid.classList.toggle("single", layout === "single");
  el.layoutQuad.classList.toggle("active", layout === "quad");
  el.layoutCamera.classList.toggle("active", layout === "single");
  updateViewportHint(
    layout === "quad"
      ? "4 View layout active: Camera, Top, Front, and Side panels."
      : "Camera layout active."
  );
  draw();
  requestAnimationFrame(draw);
}

function updateViewportHint(message = null) {
  if (message) {
    el.viewportHint.textContent = message;
    return;
  }
  const brush = currentBrush();
  el.viewportHint.textContent =
    state.tool === "draw"
      ? `Draw ${brush.kind}: drag Camera, Top, Front, or Side. Orthographic work planes are set in Create.`
      : state.tool === "place"
        ? "Place: click Camera or an ortho view. Right-click can place at cursor."
        : state.tool === "move"
          ? "Move: drag selected props. F focuses, Ctrl+D duplicates."
          : "Select: click props. FPS right-drag looks, WASD moves, Q/E changes height, right-click opens commands.";
}

function describeBrushPreview(start, current, viewName = state.drawBrush.viewName) {
  const brush = currentBrush();
  const dimensions = drawBrushDimensions(start, current, brush, viewName);
  const [width, height, depth] = dimensions.scale;
  const plane = dimensions.viewName === "top"
    ? `Y ${formatNumber(brush.groundY)}`
    : dimensions.viewName === "front"
      ? `Z ${formatNumber(brush.frontZ)}`
      : `X ${formatNumber(brush.sideX)}`;
  return `${brush.kind} ${formatNumber(width)} × ${formatNumber(height)} × ${formatNumber(depth)} from ${titleCase(dimensions.viewName)} (${plane})`;
}

function showContextMenu(event, context) {
  event.preventDefault();
  if (state.suppressContextMenu || performance.now() < state.suppressContextMenuUntil) {
    state.suppressContextMenu = false;
    return;
  }
  state.contextMenu = context;
  const menu = el.contextMenu;
  for (const button of menu.querySelectorAll("button")) {
    const action = button.dataset.action;
    button.disabled = contextActionDisabled(action, context);
  }
  menu.hidden = false;
  const width = menu.offsetWidth || 190;
  const height = menu.offsetHeight || 260;
  menu.style.left = `${Math.min(event.clientX, window.innerWidth - width - 8)}px`;
  menu.style.top = `${Math.min(event.clientY, window.innerHeight - height - 8)}px`;
}

function hideContextMenu() {
  el.contextMenu.hidden = true;
}

function contextActionDisabled(action, context) {
  const hasLevel = !!state.level && !state.levelLoading;
  const hasSelection = selectionIndices().length > 0;
  const hasWorld = !!context.world;
  if (action === "undo") return state.history.past.length === 0;
  if (action === "redo") return state.history.future.length === 0;
  if (action === "selectHere") return context.propIndex == null;
  if (action === "placeHere") return !hasLevel || !hasWorld || !state.selectedTemplate;
  if (action === "drawHere") return !hasLevel || !hasWorld;
  if (["addPathPointHere", "addEventTriggerHere"].includes(action)) return !hasLevel || !hasWorld;
  if (action === "moveSelectedHere") return !hasLevel || !hasWorld || !hasSelection;
  if (action === "pasteHere") return !hasLevel || !hasWorld || state.clipboardProps.length === 0;
  if (["selectAll", "invertSelection"].includes(action)) return !state.level?.props?.length;
  if (["copySelected", "duplicateSelected", "focusSelected", "deleteSelected", "clearSelection"].includes(action)) {
    return !hasSelection;
  }
  return false;
}

function runContextAction(action) {
  const context = state.contextMenu;
  hideContextMenu();
  if (action === "selectHere") {
    setSelection(context.propIndex == null ? [] : [context.propIndex], context.propIndex);
    renderAll();
    updateViewportHint(
      context.propIndex == null
        ? "Nothing selected."
        : `Selected ${propLabel(state.level.props[context.propIndex], context.propIndex)}.`
    );
  } else if (action === "placeHere") {
    const prop = addPropAt(context.world);
    if (prop) updateViewportHint(`Placed ${placementLabel(prop)}.`);
  } else if (action === "drawHere") {
    createBrushAt(context.world, context.viewName);
  } else if (action === "addPathPointHere") {
    addPathPointAt(context.world);
  } else if (action === "addEventTriggerHere") {
    addEventTriggerAt(context.world);
  } else if (action === "moveSelectedHere") {
    moveSelectedTo(context.world);
  } else if (action === "pasteHere") {
    pasteCopied(context.world);
  } else if (action === "selectAll") {
    selectAllProps();
  } else if (action === "invertSelection") {
    invertSelection();
  } else {
    performEditorAction(action);
  }
}

function createBrushAt(world, requestedView = "top") {
  if (!state.level) {
    updateViewportHint("Load a level before drawing geometry.");
    return null;
  }
  if (!world) {
    return null;
  }
  const viewName = normalizeDrawView(requestedView);
  const view = orthoView(viewName);
  const center = pointArray(world).map((value) => snap(value));
  const start = [...center];
  const current = [...center];
  start[view.h] -= 2;
  start[view.v] -= 2;
  current[view.h] += 2;
  current[view.v] += 2;
  state.drawBrush.viewName = viewName;
  state.drawBrush.start = { x: start[0], y: start[1], z: start[2] };
  state.drawBrush.current = { x: current[0], y: current[1], z: current[2] };
  const prop = createDrawGeometry();
  state.drawBrush.start = null;
  state.drawBrush.current = null;
  if (prop) {
    updateViewportHint(`Created ${propLabel(prop, state.selectedProp)} from context menu.`);
  }
  draw();
  return prop;
}

function cameraContext(clientX, clientY) {
  return {
    viewName: "camera",
    world: groundHit(clientX, clientY),
    propIndex: pickProp(clientX, clientY),
  };
}

function orthoContext(canvas, viewName, clientX, clientY) {
  return {
    viewName,
    world: orthoWorldFromEvent(canvas, viewName, clientX, clientY),
    propIndex: pickPropOrtho(canvas, viewName, clientX, clientY),
  };
}

function performEditorAction(action, options = {}) {
  if (action === "toolSelect") setTool("select");
  else if (action === "toolMove") setTool("move");
  else if (action === "toolPlace") setTool("place");
  else if (action === "toolDraw") setTool("draw");
  else if (action === "undo") undoHistory();
  else if (action === "redo" || action === "redoAlt") redoHistory();
  else if (action === "validate") {
    setWorkspaceTab("validation");
    validateLevel(true);
  } else if (action === "save") saveLevel();
  else if (action === "deleteSelected") deleteSelected();
  else if (action === "duplicateSelected") duplicateSelected();
  else if (action === "copySelected") copySelected();
  else if (action === "pasteSelected") pasteCopied(options.world);
  else if (action === "selectAll") selectAllProps();
  else if (action === "invertSelection") invertSelection();
  else if (action === "nudgeLeft") nudgeSelection([-1, 0, 0]);
  else if (action === "nudgeRight") nudgeSelection([1, 0, 0]);
  else if (action === "nudgeForward") nudgeSelection([0, 0, -1]);
  else if (action === "nudgeBackward") nudgeSelection([0, 0, 1]);
  else if (action === "nudgeUp") nudgeSelection([0, 1, 0]);
  else if (action === "nudgeDown") nudgeSelection([0, -1, 0]);
  else if (action === "clearSelection") clearSelection();
  else if (action === "focusSelected") focusSelected();
  else if (action === "resetCamera") {
    resetCameraToLevel();
    updateViewportHint("Camera reset to the current level spawn.");
  } else if (action === "toggleLayout") {
    setViewLayout(state.viewLayout === "quad" ? "single" : "quad");
  } else if (action === "cancel") {
    hideContextMenu();
    cancelActiveInteraction();
  }
}

function cancelActiveInteraction(message = "Cancelled current editor action.") {
  const drag = state.drag;
  if (["move", "move-y", "ortho-move"].includes(drag?.kind)) {
    for (const [index, position] of drag.positions || []) {
      if (state.level?.props?.[index]) {
        state.level.props[index].position = [...position];
      }
    }
    state.history.transactionChanged = false;
    endHistoryTransaction();
    state.dirty = state.history.saved == null || snapshotLevel() !== state.history.saved;
  }
  if (drag?.kind === "ortho-marquee") {
    setSelection(drag.baseSelection || [], (drag.baseSelection || []).at(-1));
  }
  state.drawBrush.start = null;
  state.drawBrush.current = null;
  state.drag = null;
  renderAll();
  updateViewportHint(message);
}

function draw() {
  state.needsDraw = true;
}

function pickProp(clientX, clientY) {
  if (!state.level) {
    return null;
  }
  const ray = viewportRay(clientX, clientY);
  let best = null;
  let bestDistance = Infinity;
  state.level.props.forEach((prop, index) => {
    const bounds = propBounds(prop);
    const pad = 0.35;
    const hit = intersectRayAabb(
      ray.origin,
      ray.direction,
      bounds.min.map((value) => value - pad),
      bounds.max.map((value) => value + pad)
    );
    if (hit != null && hit < bestDistance) {
      best = index;
      bestDistance = hit;
    }
  });
  return best;
}

function groundHit(clientX, clientY, groundY = currentBrush().groundY) {
  const ray = viewportRay(clientX, clientY);
  const dy = ray.direction[1];
  if (Math.abs(dy) < 0.0001) {
    return null;
  }
  const t = (groundY - ray.origin[1]) / dy;
  if (t <= 0) {
    return null;
  }
  return {
    x: ray.origin[0] + ray.direction[0] * t,
    y: groundY,
    z: ray.origin[2] + ray.direction[2] * t,
  };
}

function viewportRay(clientX, clientY) {
  const rect = el.viewport.getBoundingClientRect();
  const ndcX = ((clientX - rect.left) / rect.width) * 2 - 1;
  const ndcY = 1 - ((clientY - rect.top) / rect.height) * 2;
  const aspect = rect.width / Math.max(1, rect.height);
  const tan = Math.tan((60 * Math.PI / 180) * 0.5);
  const cameraDir = normalize3([ndcX * aspect * tan, ndcY * tan, -1]);
  const basis = cameraBasis();
  const direction = normalize3(add3(
    add3(scale3(basis.right, cameraDir[0]), scale3(basis.up, cameraDir[1])),
    scale3(basis.forward, -cameraDir[2])
  ));
  return { origin: [...state.camera.position], direction };
}

function intersectRayAabb(origin, direction, min, max) {
  let tMin = 0;
  let tMax = Infinity;
  for (let axis = 0; axis < 3; axis += 1) {
    if (Math.abs(direction[axis]) < 0.000001) {
      if (origin[axis] < min[axis] || origin[axis] > max[axis]) {
        return null;
      }
      continue;
    }
    let near = (min[axis] - origin[axis]) / direction[axis];
    let far = (max[axis] - origin[axis]) / direction[axis];
    if (near > far) {
      [near, far] = [far, near];
    }
    tMin = Math.max(tMin, near);
    tMax = Math.min(tMax, far);
    if (tMin > tMax) {
      return null;
    }
  }
  return tMin >= 0 ? tMin : tMax >= 0 ? tMax : null;
}

function renderOrthoViews() {
  drawOrthoView(el.topView, "top");
  drawOrthoView(el.frontView, "front");
  drawOrthoView(el.sideView, "side");
}

function drawOrthoView(canvas, viewName) {
  if (!canvas || canvas.offsetParent == null) {
    return;
  }
  const frame = resizeCanvas2d(canvas);
  const { ctx, width, height } = frame;
  const view = orthoView(viewName);
  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = "#090c0e";
  ctx.fillRect(0, 0, width, height);
  drawOrthoGrid(ctx, view, width, height);
  drawOrthoSpawn(ctx, view, width, height);
  (state.level?.props || []).forEach((prop, index) => {
    drawOrthoProp(ctx, view, width, height, prop, index);
  });
  drawOrthoSystems(ctx, view, width, height);
  drawOrthoBrushPreview(ctx, view, width, height);
  drawOrthoMarquee(ctx, viewName);
}

function resizeCanvas2d(canvas) {
  const rect = canvas.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  const width = Math.max(1, Math.floor(rect.width));
  const height = Math.max(1, Math.floor(rect.height));
  const pixelWidth = Math.max(1, Math.floor(rect.width * dpr));
  const pixelHeight = Math.max(1, Math.floor(rect.height * dpr));
  if (canvas.width !== pixelWidth || canvas.height !== pixelHeight) {
    canvas.width = pixelWidth;
    canvas.height = pixelHeight;
  }
  const ctx = canvas.getContext("2d");
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  return { ctx, width, height };
}

function drawOrthoGrid(ctx, view, width, height) {
  const zoom = state.ortho.zoom;
  const minH = state.ortho.center[view.h] - width / (2 * zoom);
  const maxH = state.ortho.center[view.h] + width / (2 * zoom);
  const minV = state.ortho.center[view.v] - height / (2 * zoom);
  const maxV = state.ortho.center[view.v] + height / (2 * zoom);
  const step = gridStep(zoom);
  ctx.lineWidth = 1;
  ctx.strokeStyle = "#1f2a31";
  ctx.beginPath();
  for (let h = Math.floor(minH / step) * step; h <= maxH; h += step) {
    const x = orthoScreenFromAxes(view, h, 0, width, height).x;
    ctx.moveTo(x, 0);
    ctx.lineTo(x, height);
  }
  for (let v = Math.floor(minV / step) * step; v <= maxV; v += step) {
    const y = orthoScreenFromAxes(view, 0, v, width, height).y;
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
  }
  ctx.stroke();

  ctx.strokeStyle = view.h === 0 ? "#486f9f" : "#9f5348";
  ctx.beginPath();
  const axisX = orthoScreenFromAxes(view, 0, 0, width, height).x;
  ctx.moveTo(axisX, 0);
  ctx.lineTo(axisX, height);
  ctx.stroke();

  ctx.strokeStyle = view.v === 1 ? "#7ca66a" : "#9f5348";
  ctx.beginPath();
  const axisY = orthoScreenFromAxes(view, 0, 0, width, height).y;
  ctx.moveTo(0, axisY);
  ctx.lineTo(width, axisY);
  ctx.stroke();
}

function gridStep(zoom) {
  if (zoom >= 10) return 1;
  if (zoom >= 4) return 5;
  if (zoom >= 1.5) return 10;
  return 25;
}

function drawOrthoSpawn(ctx, view, width, height) {
  if (!state.level) {
    return;
  }
  const spawn = vector(state.level.player_spawn);
  const point = worldToOrthoScreen(view, spawn, width, height);
  ctx.strokeStyle = "#ffffff";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(point.x - 7, point.y);
  ctx.lineTo(point.x + 7, point.y);
  ctx.moveTo(point.x, point.y - 7);
  ctx.lineTo(point.x, point.y + 7);
  ctx.stroke();
}

function drawOrthoProp(ctx, view, width, height, prop, index) {
  const bounds = propBounds(prop);
  const min = worldToOrthoScreen(view, bounds.min, width, height);
  const max = worldToOrthoScreen(view, bounds.max, width, height);
  const x = Math.min(min.x, max.x);
  const y = Math.min(min.y, max.y);
  const boxWidth = Math.max(6, Math.abs(max.x - min.x));
  const boxHeight = Math.max(6, Math.abs(max.y - min.y));
  const selected = selectionContains(index);
  ctx.fillStyle = hexToRgba(colorForKind(kindForProp(prop)), selected ? 0.68 : 0.42);
  ctx.strokeStyle = index === state.selectedProp ? "#ffffff" : selected ? "#69e6d1" : "#11181d";
  ctx.lineWidth = selected ? 2 : 1;
  ctx.fillRect(x, y, boxWidth, boxHeight);
  ctx.strokeRect(x, y, boxWidth, boxHeight);
}

function drawOrthoMarquee(ctx, viewName) {
  if (state.drag?.kind !== "ortho-marquee" || state.drag.viewName !== viewName) {
    return;
  }
  const x = Math.min(state.drag.startX, state.drag.currentX);
  const y = Math.min(state.drag.startY, state.drag.currentY);
  const width = Math.abs(state.drag.currentX - state.drag.startX);
  const height = Math.abs(state.drag.currentY - state.drag.startY);
  ctx.fillStyle = "rgba(105, 230, 209, 0.12)";
  ctx.strokeStyle = "#8cf6e0";
  ctx.lineWidth = 1;
  ctx.setLineDash([5, 4]);
  ctx.fillRect(x, y, width, height);
  ctx.strokeRect(x, y, width, height);
  ctx.setLineDash([]);
}

function drawOrthoSystems(ctx, view, width, height) {
  if (!state.level) {
    return;
  }
  ctx.lineWidth = 2;
  for (const path of state.level.paths || []) {
    const waypoints = (path.waypoints || []).map(vector);
    if (waypoints.length === 0) {
      continue;
    }
    ctx.strokeStyle = "#f0bf4c";
    ctx.fillStyle = "#f0bf4c";
    ctx.beginPath();
    waypoints.forEach((point, index) => {
      const screen = worldToOrthoScreen(view, point, width, height);
      if (index === 0) ctx.moveTo(screen.x, screen.y);
      else ctx.lineTo(screen.x, screen.y);
    });
    ctx.stroke();
    for (const point of waypoints) {
      const screen = worldToOrthoScreen(view, point, width, height);
      ctx.beginPath();
      ctx.arc(screen.x, screen.y, 4, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  for (const event of state.level.events || []) {
    const trigger = event.trigger || {};
    const position = vector(trigger.position);
    const screen = worldToOrthoScreen(view, position, width, height);
    ctx.strokeStyle = "#69e6d1";
    ctx.fillStyle = "rgba(105, 230, 209, 0.12)";
    ctx.beginPath();
    if (view.plane === 1) {
      ctx.arc(screen.x, screen.y, Math.max(5, (trigger.radius || 2.5) * state.ortho.zoom), 0, Math.PI * 2);
    } else {
      ctx.rect(screen.x - 5, screen.y - 5, 10, 10);
    }
    ctx.fill();
    ctx.stroke();
  }
}

function drawOrthoBrushPreview(ctx, view, width, height) {
  if (!state.level || !state.drawBrush.start || !state.drawBrush.current) {
    return;
  }
  const prop = drawBrushProp(state.drawBrush.start, state.drawBrush.current);
  const bounds = propBounds(prop);
  const min = worldToOrthoScreen(view, bounds.min, width, height);
  const max = worldToOrthoScreen(view, bounds.max, width, height);
  const x = Math.min(min.x, max.x);
  const y = Math.min(min.y, max.y);
  const boxWidth = Math.max(6, Math.abs(max.x - min.x));
  const boxHeight = Math.max(6, Math.abs(max.y - min.y));
  ctx.fillStyle = "rgba(64, 255, 220, 0.28)";
  ctx.strokeStyle = "#b8fff5";
  ctx.lineWidth = 2;
  ctx.fillRect(x, y, boxWidth, boxHeight);
  ctx.strokeRect(x, y, boxWidth, boxHeight);
}

function orthoScreenFromAxes(view, h, v, width, height) {
  return {
    x: width / 2 + (h - state.ortho.center[view.h]) * state.ortho.zoom,
    y: height / 2 - (v - state.ortho.center[view.v]) * state.ortho.zoom,
  };
}

function worldToOrthoScreen(view, point, width, height) {
  return orthoScreenFromAxes(view, point[view.h], point[view.v], width, height);
}

function orthoWorldFromEvent(canvas, viewName, clientX, clientY) {
  const view = orthoView(viewName);
  const rect = canvas.getBoundingClientRect();
  const point = [...state.ortho.center];
  point[view.h] = state.ortho.center[view.h] + (clientX - rect.left - rect.width / 2) / state.ortho.zoom;
  point[view.v] = state.ortho.center[view.v] - (clientY - rect.top - rect.height / 2) / state.ortho.zoom;
  if (viewName === "top") {
    point[1] = currentBrush().groundY;
  } else if (viewName === "front") {
    point[2] = currentBrush().frontZ;
  } else if (viewName === "side") {
    point[0] = currentBrush().sideX;
  }
  return { x: point[0], y: point[1], z: point[2], point };
}

function pickPropOrtho(canvas, viewName, clientX, clientY) {
  if (!state.level) {
    return null;
  }
  const view = orthoView(viewName);
  const rect = canvas.getBoundingClientRect();
  const x = clientX - rect.left;
  const y = clientY - rect.top;
  let best = null;
  let bestDistance = Infinity;
  state.level.props.forEach((prop, index) => {
    const bounds = propBounds(prop);
    const min = worldToOrthoScreen(view, bounds.min, rect.width, rect.height);
    const max = worldToOrthoScreen(view, bounds.max, rect.width, rect.height);
    const left = Math.min(min.x, max.x) - 3;
    const right = Math.max(min.x, max.x) + 3;
    const top = Math.min(min.y, max.y) - 3;
    const bottom = Math.max(min.y, max.y) + 3;
    const inside = x >= left && x <= right && y >= top && y <= bottom;
    if (!inside) {
      return;
    }
    const center = worldToOrthoScreen(view, bounds.center, rect.width, rect.height);
    const distance = Math.hypot(x - center.x, y - center.y);
    if (distance < bestDistance) {
      best = index;
      bestDistance = distance;
    }
  });
  return best;
}

function orthoView(viewName) {
  return {
    top: { h: 0, v: 2, plane: 1 },
    front: { h: 0, v: 1, plane: 2 },
    side: { h: 2, v: 1, plane: 0 },
  }[viewName];
}

function orthoCanvas(viewName) {
  return {
    top: el.topView,
    front: el.frontView,
    side: el.sideView,
  }[viewName];
}

function installOrthoEvents(canvas, viewName) {
  canvas.addEventListener("contextmenu", (event) => {
    showContextMenu(event, orthoContext(canvas, viewName, event.clientX, event.clientY));
  });
  canvas.addEventListener("wheel", (event) => {
    event.preventDefault();
    const before = orthoWorldFromEvent(canvas, viewName, event.clientX, event.clientY).point;
    const factor = event.deltaY < 0 ? 1.12 : 0.88;
    state.ortho.zoom = clamp(state.ortho.zoom * factor, 0.4, 24);
    const after = orthoWorldFromEvent(canvas, viewName, event.clientX, event.clientY).point;
    const view = orthoView(viewName);
    state.ortho.center[view.h] += before[view.h] - after[view.h];
    state.ortho.center[view.v] += before[view.v] - after[view.v];
    draw();
  });

  canvas.addEventListener("pointerdown", (event) => {
    canvas.setPointerCapture(event.pointerId);
    if (event.button === 2 || event.button === 1) {
      state.drag = {
        kind: "ortho-pan",
        viewName,
        startX: event.clientX,
        startY: event.clientY,
        center: [...state.ortho.center],
        moved: false,
      };
      return;
    }

    if ((!state.level || state.levelLoading) && ["place", "draw"].includes(state.tool)) {
      updateViewportHint("Load a level before placing or drawing content.");
      return;
    }

    const world = orthoWorldFromEvent(canvas, viewName, event.clientX, event.clientY);
    if (state.tool === "place") {
      const prop = addPropAt(world);
      if (prop) {
        updateViewportHint(`Placed ${placementLabel(prop)} from ${viewName} view.`);
      }
      return;
    }

    if (state.tool === "draw") {
      state.drawBrush.viewName = viewName;
      state.drawBrush.start = snappedPoint(world);
      state.drawBrush.current = { ...state.drawBrush.start };
      state.drag = { kind: "draw", viewName };
      updateViewportHint(`Drawing ${describeBrushPreview(state.drawBrush.start, state.drawBrush.current, viewName)}.`);
      draw();
      return;
    }

    const picked = pickPropOrtho(canvas, viewName, event.clientX, event.clientY);
    const additive = event.ctrlKey || event.metaKey;
    if (state.tool === "select" && picked == null) {
      const rect = canvas.getBoundingClientRect();
      state.drag = {
        kind: "ortho-marquee",
        viewName,
        startX: event.clientX - rect.left,
        startY: event.clientY - rect.top,
        currentX: event.clientX - rect.left,
        currentY: event.clientY - rect.top,
        additive,
        baseSelection: additive ? selectionIndices() : [],
      };
      if (!additive) {
        resetSelection();
      }
      renderAll();
      updateViewportHint(`Drag to box-select props in ${viewName} view.`);
      return;
    }

    if (picked != null) {
      if (state.tool === "select") {
        selectProp(picked, { additive, toggle: additive, range: event.shiftKey });
      } else if (!selectionContains(picked)) {
        setSelection([picked], picked);
      }
    } else if (!additive) {
      resetSelection();
    }

    if (state.tool === "move" && picked != null) {
      beginHistoryTransaction();
      state.drag = {
        kind: "ortho-move",
        viewName,
        startPoint: [...world.point],
        indices: selectionIndices(),
        positions: selectionIndices().map((index) => [index, vector(state.level.props[index].position)]),
      };
    }
    renderAll();
    updateViewportHint(
      picked == null
        ? `Nothing selected in ${viewName} view.`
        : `${selectionIndices().length} selected in ${viewName} view.`
    );
  });

  canvas.addEventListener("pointermove", (event) => {
    if (!state.drag || state.drag.viewName !== viewName) {
      return;
    }
    const view = orthoView(viewName);
    if (state.drag.kind === "ortho-pan") {
      if (Math.hypot(event.clientX - state.drag.startX, event.clientY - state.drag.startY) > 4) {
        state.drag.moved = true;
        state.suppressContextMenu = true;
        state.suppressContextMenuUntil = performance.now() + 350;
      }
      state.ortho.center = [...state.drag.center];
      state.ortho.center[view.h] -= (event.clientX - state.drag.startX) / state.ortho.zoom;
      state.ortho.center[view.v] += (event.clientY - state.drag.startY) / state.ortho.zoom;
      draw();
      return;
    }

    const world = orthoWorldFromEvent(canvas, viewName, event.clientX, event.clientY);
    if (state.drag.kind === "ortho-marquee") {
      const rect = canvas.getBoundingClientRect();
      state.drag.currentX = event.clientX - rect.left;
      state.drag.currentY = event.clientY - rect.top;
      draw();
      return;
    }
    if (state.drag.kind === "draw") {
      state.drawBrush.current = snappedPoint(world);
      updateViewportHint(`Drawing ${describeBrushPreview(state.drawBrush.start, state.drawBrush.current, viewName)}.`);
      draw();
      return;
    }

    if (state.drag.kind === "ortho-move") {
      const delta = sub3(world.point, state.drag.startPoint);
      for (const [index, initial] of state.drag.positions) {
        const prop = state.level?.props[index];
        if (!prop) {
          continue;
        }
        const next = [...initial];
        if (axisAllows(view.h)) next[view.h] = snap(initial[view.h] + delta[view.h]);
        if (axisAllows(view.v)) next[view.v] = snap(initial[view.v] + delta[view.v]);
        prop.position = next;
      }
      markDirty();
      renderObjects();
      updateViewportHint(
        `Moving ${state.drag.indices.length} ${state.drag.indices.length === 1 ? "prop" : "props"}.`
      );
      draw();
    }
  });
}

function finishOrthoMarquee(drag) {
  const canvas = orthoCanvas(drag.viewName);
  if (!canvas || !state.level) {
    return;
  }
  const left = Math.min(drag.startX, drag.currentX);
  const right = Math.max(drag.startX, drag.currentX);
  const top = Math.min(drag.startY, drag.currentY);
  const bottom = Math.max(drag.startY, drag.currentY);
  const moved = right - left >= 4 || bottom - top >= 4;
  if (!moved) {
    setSelection(drag.baseSelection || [], (drag.baseSelection || []).at(-1));
    renderAll();
    updateViewportHint(drag.baseSelection?.length ? "Selection unchanged." : "Selection cleared.");
    return;
  }

  const view = orthoView(drag.viewName);
  const rect = canvas.getBoundingClientRect();
  const hits = [];
  state.level.props.forEach((prop, index) => {
    const bounds = propBounds(prop);
    const min = worldToOrthoScreen(view, bounds.min, rect.width, rect.height);
    const max = worldToOrthoScreen(view, bounds.max, rect.width, rect.height);
    const propLeft = Math.min(min.x, max.x);
    const propRight = Math.max(min.x, max.x);
    const propTop = Math.min(min.y, max.y);
    const propBottom = Math.max(min.y, max.y);
    if (propRight >= left && propLeft <= right && propBottom >= top && propTop <= bottom) {
      hits.push(index);
    }
  });
  const combined = drag.additive ? [...drag.baseSelection, ...hits] : hits;
  setSelection(combined, hits.at(-1) ?? (drag.baseSelection || []).at(-1));
  renderAll();
  updateViewportHint(`${selectionIndices().length} props selected.`);
}

function installEvents() {
  el.reconnectEditor.addEventListener("click", reconnectEditor);
  window.addEventListener("hashchange", () => {
    const fragment = new URLSearchParams(window.location.hash.slice(1));
    const token = fragment.get("token");
    if (token && token !== editorToken) {
      sessionStorage.setItem("cenotaphEditorToken", token);
      window.location.reload();
    }
  });
  el.refreshProject.addEventListener("click", refreshProject);
  el.newLevel.addEventListener("click", newLevel);
  el.duplicateLevel.addEventListener("click", duplicateLevel);
  el.undoAction.addEventListener("click", undoHistory);
  el.redoAction.addEventListener("click", redoHistory);
  el.validateLevel.addEventListener("click", () => {
    setWorkspaceTab("validation");
    validateLevel(true);
  });
  el.saveLevel.addEventListener("click", saveLevel);
  el.objectFilter.addEventListener("input", renderObjects);
  el.deleteSelected.addEventListener("click", deleteSelected);
  el.toolSelect.addEventListener("click", () => setTool("select"));
  el.toolMove.addEventListener("click", () => setTool("move"));
  el.toolPlace.addEventListener("click", () => setTool("place"));
  el.toolDraw.addEventListener("click", () => setTool("draw"));
  el.layoutQuad.addEventListener("click", () => setViewLayout("quad"));
  el.layoutCamera.addEventListener("click", () => setViewLayout("single"));
  for (const button of el.workspaceTabs) {
    button.addEventListener("click", () => setWorkspaceTab(button.dataset.editorTab));
  }
  el.transformAxis.addEventListener("change", () => {
    state.transform.axis = el.transformAxis.value;
    renderBrushControls();
    renderInspector();
    updateViewportHint(`Transform axis set to ${state.transform.axis === "all" ? "XYZ" : state.transform.axis.toUpperCase()}.`);
  });
  el.snapToggle.addEventListener("click", () => {
    state.transform.snap = !state.transform.snap;
    renderBrushControls();
    updateViewportHint(`Grid snapping ${state.transform.snap ? "enabled" : "disabled"}.`);
  });
  el.assetKindFilter.addEventListener("change", renderAssetBrowser);
  el.assetFilter.addEventListener("input", renderAssetBrowser);
  el.prefabFilter.addEventListener("input", renderPrefabs);
  el.prefabName.addEventListener("input", () => {
    if (el.prefabId.dataset.manual !== "true") {
      el.prefabId.value = sanitizeLevelId(el.prefabName.value);
    }
    renderAvailabilityControls();
  });
  el.prefabId.addEventListener("input", () => {
    el.prefabId.dataset.manual = el.prefabId.value.trim() ? "true" : "false";
    if (!el.prefabId.value.trim()) {
      el.prefabId.value = sanitizeLevelId(el.prefabName.value);
    }
    renderAvailabilityControls();
  });
  el.createPrefab.addEventListener("click", createPrefabFromSelection);
  el.deletePrefab.addEventListener("click", deleteSelectedPrefab);
  el.resetKeybindings.addEventListener("click", resetKeybindings);
  el.contextMenu.addEventListener("click", (event) => {
    const button = event.target.closest("button[data-action]");
    if (!button || button.disabled) {
      return;
    }
    runContextAction(button.dataset.action);
  });
  window.addEventListener("pointerdown", (event) => {
    if (!el.contextMenu.hidden && !el.contextMenu.contains(event.target)) {
      hideContextMenu();
    }
  });
  el.resetCamera.addEventListener("click", () => {
    resetCameraToLevel();
    updateViewportHint("Camera reset to the current level spawn.");
    draw();
  });
  for (const input of [
    el.brushKind,
    el.brushHeight,
    el.brushThickness,
    el.brushDirection,
    el.brushSegments,
    el.brushSteps,
    el.terrainResolution,
    el.terrainRelief,
    el.terrainSeed,
    el.brushGround,
    el.brushFrontZ,
    el.brushSideX,
    el.gridSize,
  ]) {
    input.addEventListener("input", () => {
      currentBrush();
      state.gridSize = readNumberInput(el.gridSize, 1, 0.1);
      renderBrushControls();
      updateViewportHint();
      draw();
    });
  }
  document.addEventListener("focusin", (event) => {
    if (event.target.closest?.(".right-panel") && event.target.matches?.("input, select, textarea")) {
      event.target.dataset.historyTransaction = "true";
      beginHistoryTransaction();
    }
  });
  document.addEventListener("focusout", (event) => {
    if (event.target.dataset?.historyTransaction === "true") {
      delete event.target.dataset.historyTransaction;
      endHistoryTransaction();
    }
  });
  window.addEventListener("resize", draw);
  window.addEventListener("beforeunload", (event) => {
    if (!state.dirty) {
      return;
    }
    persistLocalDraft();
    event.preventDefault();
    event.returnValue = "";
  });
  installOrthoEvents(el.topView, "top");
  installOrthoEvents(el.frontView, "front");
  installOrthoEvents(el.sideView, "side");

  el.viewport.addEventListener("wheel", (event) => {
    event.preventDefault();
    const forward = cameraBasis().forward;
    const amount = event.deltaY < 0 ? 4 : -4;
    state.camera.position = add3(state.camera.position, scale3(forward, amount));
    draw();
  });

  el.viewport.addEventListener("pointerdown", (event) => {
    el.viewport.setPointerCapture(event.pointerId);
    if (event.button === 2 || event.button === 1) {
      state.drag = {
        kind: "look",
        startX: event.clientX,
        startY: event.clientY,
        yaw: state.camera.yaw,
        pitch: state.camera.pitch,
        moved: false,
      };
      return;
    }
    if ((!state.level || state.levelLoading) && ["place", "draw"].includes(state.tool)) {
      updateViewportHint("Load a level before placing or drawing content.");
      return;
    }
    const world = groundHit(event.clientX, event.clientY);
    if (state.tool === "place") {
      if (world) {
        const prop = addPropAt(world);
        if (prop) {
          updateViewportHint(`Placed ${placementLabel(prop)}. Validate before saving.`);
        }
      } else {
        updateViewportHint("No ground hit. Aim at the grid plane before placing.");
      }
      return;
    }
    if (state.tool === "draw") {
      if (world) {
        state.drawBrush.viewName = "top";
        state.drawBrush.start = snappedPoint(world);
        state.drawBrush.current = { ...state.drawBrush.start };
        state.drag = { kind: "draw" };
        updateViewportHint(`Drawing ${describeBrushPreview(state.drawBrush.start, state.drawBrush.current, "top")}.`);
        draw();
      } else {
        updateViewportHint("No ground hit. Aim at the grid plane before drawing.");
      }
      return;
    }
    const picked = pickProp(event.clientX, event.clientY);
    const additive = event.ctrlKey || event.metaKey;
    if (picked != null) {
      if (state.tool === "select") {
        selectProp(picked, { additive, toggle: additive });
      } else if (!selectionContains(picked)) {
        setSelection([picked], picked);
      }
    } else if (!additive) {
      resetSelection();
    }
    if (state.tool === "move" && picked != null) {
      const primary = state.level.props[state.selectedProp];
      const groundY = vector(primary.position)[1];
      const hit = groundHit(event.clientX, event.clientY, groundY);
      beginHistoryTransaction();
      state.drag = {
        kind: state.transform.axis === "y" ? "move-y" : "move",
        groundY,
        startX: event.clientX,
        startY: event.clientY,
        startWorld: hit ? [hit.x, groundY, hit.z] : null,
        indices: selectionIndices(),
        positions: selectionIndices().map((index) => [index, vector(state.level.props[index].position)]),
      };
    }
    renderAll();
    updateViewportHint(
      picked == null
        ? "Nothing selected."
        : `${selectionIndices().length} selected.`
    );
  });

  el.viewport.addEventListener("pointermove", (event) => {
    if (!state.drag) {
      return;
    }
    if (state.drag.kind === "look") {
      if (Math.hypot(event.clientX - state.drag.startX, event.clientY - state.drag.startY) > 4) {
        state.drag.moved = true;
        state.suppressContextMenu = true;
        state.suppressContextMenuUntil = performance.now() + 350;
      }
      state.camera.yaw = state.drag.yaw + (event.clientX - state.drag.startX) * 0.006;
      state.camera.pitch = clamp(
        state.drag.pitch - (event.clientY - state.drag.startY) * 0.006,
        -1.45,
        1.2
      );
      draw();
    } else if (state.drag.kind === "draw") {
      const hit = groundHit(event.clientX, event.clientY);
      if (hit) {
        state.drawBrush.current = snappedPoint(hit);
        updateViewportHint(`Drawing ${describeBrushPreview(state.drawBrush.start, state.drawBrush.current, "top")}.`);
      }
      draw();
    } else if (state.drag.kind === "move") {
      const world = groundHit(event.clientX, event.clientY, state.drag.groundY);
      if (!world || !state.drag.startWorld) {
        return;
      }
      const delta = [world.x - state.drag.startWorld[0], 0, world.z - state.drag.startWorld[2]];
      for (const [index, initial] of state.drag.positions) {
        const prop = state.level?.props[index];
        if (!prop) continue;
        const next = [...initial];
        if (axisAllows(0)) next[0] = snap(initial[0] + delta[0]);
        if (axisAllows(2)) next[2] = snap(initial[2] + delta[2]);
        prop.position = next;
      }
      markDirty();
      renderObjects();
      updateViewportHint(`Moving ${state.drag.indices.length} ${state.drag.indices.length === 1 ? "prop" : "props"}.`);
      draw();
    } else if (state.drag.kind === "move-y") {
      const deltaY = -(event.clientY - state.drag.startY) / Math.max(1, state.ortho.zoom);
      for (const [index, initial] of state.drag.positions) {
        const prop = state.level?.props[index];
        if (!prop) continue;
        const next = [...initial];
        next[1] = snap(initial[1] + deltaY);
        prop.position = next;
      }
      markDirty();
      renderObjects();
      updateViewportHint(`Moving ${state.drag.indices.length} ${state.drag.indices.length === 1 ? "prop" : "props"} on Y.`);
      draw();
    }
  });

  const finishPointerInteraction = (event) => {
    if (event.type === "pointercancel") {
      cancelActiveInteraction("Pointer action cancelled.");
      return;
    }
    const drag = state.drag;
    if (drag?.kind === "draw") {
      const prop = createDrawGeometry();
      if (prop) {
        updateViewportHint(`Created ${propLabel(prop, state.selectedProp)}. Validate before saving.`);
      }
      state.drawBrush.start = null;
      state.drawBrush.current = null;
    }
    if (drag?.kind === "ortho-marquee") {
      finishOrthoMarquee(drag);
    }
    if (["move", "move-y", "ortho-move"].includes(drag?.kind)) {
      endHistoryTransaction();
      renderInspector();
    }
    if (state.suppressContextMenu) {
      window.setTimeout(() => {
        state.suppressContextMenu = false;
      }, 400);
    }
    state.drag = null;
    draw();
  };
  window.addEventListener("pointerup", finishPointerInteraction);
  window.addEventListener("pointercancel", finishPointerInteraction);

  el.viewport.addEventListener("contextmenu", (event) => {
    showContextMenu(event, cameraContext(event.clientX, event.clientY));
  });

  window.addEventListener("keydown", (event) => {
    if (
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement ||
      event.target instanceof HTMLSelectElement ||
      event.target.isContentEditable
    ) {
      return;
    }
    if (captureBinding(event)) {
      return;
    }
    const action = actionForEvent(event);
    if (!action) {
      return;
    }
    event.preventDefault();
    hideContextMenu();
    if (movementActions.has(action)) {
      state.keys.add(action);
      return;
    }
    performEditorAction(action, { world: null });
  });

  window.addEventListener("keyup", (event) => {
    const action = actionForEvent(event);
    if (action && movementActions.has(action)) {
      event.preventDefault();
      state.keys.delete(action);
    }
  });
}

function initRenderer() {
  state.renderer = new EditorRenderer(el.viewport);
  draw();
}

function animationFrame(now) {
  const dt = Math.min(0.05, (now - state.lastFrameMs) / 1000);
  state.lastFrameMs = now;
  updateCamera(dt);
  if (state.renderer && (state.needsDraw || state.renderer.needsResize())) {
    state.renderer.render();
    renderOrthoViews();
    state.needsDraw = false;
  }
  requestAnimationFrame(animationFrame);
}

function updateCamera(dt) {
  if (state.keys.size === 0) {
    return;
  }
  const basis = cameraMovementBasis();
  let move = [0, 0, 0];
  if (state.keys.has("cameraForward")) move = add3(move, basis.forward);
  if (state.keys.has("cameraBackward")) move = sub3(move, basis.forward);
  if (state.keys.has("cameraRight")) move = add3(move, basis.right);
  if (state.keys.has("cameraLeft")) move = sub3(move, basis.right);
  if (state.keys.has("cameraUp")) move = add3(move, [0, 1, 0]);
  if (state.keys.has("cameraDown")) move = sub3(move, [0, 1, 0]);
  if (length3(move) <= 0.0001) {
    return;
  }
  state.camera.position = add3(
    state.camera.position,
    scale3(normalize3(move), state.camera.speed * dt)
  );
  draw();
}

function resetCameraToLevel() {
  const spawn = state.level?.player_spawn || [0, 128, 0];
  state.camera.position = [spawn[0], spawn[1] + 12, spawn[2] + 30];
  state.camera.yaw = 0;
  state.camera.pitch = -0.35;
  state.ortho.center = [spawn[0], defaultPlacementY(), spawn[2]];
  draw();
}

function cameraBasis() {
  const yaw = state.camera.yaw;
  const pitch = state.camera.pitch;
  const cp = Math.cos(pitch);
  const forward = normalize3([
    Math.sin(yaw) * cp,
    Math.sin(pitch),
    -Math.cos(yaw) * cp,
  ]);
  const right = normalize3([Math.cos(yaw), 0, Math.sin(yaw)]);
  const up = normalize3(cross3(right, forward));
  return { forward, right, up };
}

function cameraMovementBasis() {
  const yaw = state.camera.yaw;
  return {
    forward: normalize3([Math.sin(yaw), 0, -Math.cos(yaw)]),
    right: normalize3([Math.cos(yaw), 0, Math.sin(yaw)]),
  };
}

class EditorRenderer {
  constructor(canvas) {
    this.canvas = canvas;
    this.gl = canvas.getContext("webgl", {
      antialias: true,
      alpha: false,
      depth: true,
    });
    this.width = 0;
    this.height = 0;
    if (!this.gl) {
      return;
    }

    const vertexShader = `
      attribute vec3 a_position;
      uniform mat4 u_view_proj;
      void main() {
        gl_Position = u_view_proj * vec4(a_position, 1.0);
      }
    `;
    const fragmentShader = `
      precision mediump float;
      uniform vec4 u_color;
      void main() {
        gl_FragColor = u_color;
      }
    `;
    this.program = makeProgram(this.gl, vertexShader, fragmentShader);
    this.positionLocation = this.gl.getAttribLocation(this.program, "a_position");
    this.viewProjLocation = this.gl.getUniformLocation(this.program, "u_view_proj");
    this.colorLocation = this.gl.getUniformLocation(this.program, "u_color");
    this.buffer = this.gl.createBuffer();
  }

  needsResize() {
    const rect = this.canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    return (
      this.canvas.width !== Math.max(1, Math.floor(rect.width * dpr)) ||
      this.canvas.height !== Math.max(1, Math.floor(rect.height * dpr))
    );
  }

  resize() {
    const rect = this.canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    this.canvas.width = Math.max(1, Math.floor(rect.width * dpr));
    this.canvas.height = Math.max(1, Math.floor(rect.height * dpr));
    this.width = this.canvas.width;
    this.height = this.canvas.height;
  }

  render() {
    this.resize();
    if (!this.gl) {
      const ctx = this.canvas.getContext("2d");
      ctx.fillStyle = "#0b0e10";
      ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
      ctx.fillStyle = "#edf3f6";
      ctx.font = "16px sans-serif";
      ctx.fillText("WebGL is required for the 3D level editor viewport.", 24, 32);
      return;
    }

    const gl = this.gl;
    gl.viewport(0, 0, this.width, this.height);
    gl.clearColor(0.05, 0.06, 0.07, 1);
    gl.clearDepth(1);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    const aspect = this.width / Math.max(1, this.height);
    const view = viewMatrix();
    const proj = perspectiveMatrix((60 * Math.PI) / 180, aspect, 0.1, 800);
    const viewProj = multiplyMat4(proj, view);
    gl.useProgram(this.program);
    gl.uniformMatrix4fv(this.viewProjLocation, false, new Float32Array(viewProj));

    this.drawGrid(currentBrush().groundY);
    this.drawSpawn();
    (state.level?.props || []).forEach((prop, index) => this.drawProp(prop, index));
    this.drawLevelSystems();
    this.drawBrushPreview();
  }

  drawGrid(y) {
    const size = 120;
    const step = 5;
    const vertices = [];
    for (let value = -size; value <= size; value += step) {
      vertices.push(-size, y, value, size, y, value);
      vertices.push(value, y, -size, value, y, size);
    }
    this.drawArray(this.gl.LINES, vertices, [0.14, 0.19, 0.23, 1]);
    this.drawArray(this.gl.LINES, [-size, y + 0.01, 0, size, y + 0.01, 0], [0.45, 0.68, 1, 1]);
    this.drawArray(this.gl.LINES, [0, y + 0.01, -size, 0, y + 0.01, size], [1, 0.52, 0.45, 1]);
  }

  drawSpawn() {
    if (!state.level) {
      return;
    }
    const spawn = vector(state.level.player_spawn);
    const y = spawn[1];
    const vertices = [
      spawn[0] - 1.5, y, spawn[2], spawn[0], y, spawn[2] - 1.5,
      spawn[0], y, spawn[2] - 1.5, spawn[0] + 1.5, y, spawn[2],
      spawn[0] + 1.5, y, spawn[2], spawn[0], y, spawn[2] + 1.5,
      spawn[0], y, spawn[2] + 1.5, spawn[0] - 1.5, y, spawn[2],
      spawn[0], y, spawn[2], spawn[0], y + 4, spawn[2],
    ];
    this.drawArray(this.gl.LINES, vertices, [1, 1, 1, 1]);
  }

  drawProp(prop, index) {
    const selected = selectionContains(index);
    const color = colorToVec(colorForKind(kindForProp(prop)), selected ? 0.92 : 0.62);
    const outline = index === state.selectedProp
      ? [1, 1, 1, 1]
      : selected
        ? [0.41, 0.9, 0.82, 1]
        : [0.04, 0.05, 0.06, 0.8];
    const center = vector(prop.position);
    const scale = vector(prop.scale).map((value) => Math.max(0.2, Math.abs(value)));
    const yaw = ((prop.rotation?.[1] || 0) * Math.PI) / 180;

    if (prop.brush_geometry) {
      this.drawArray(this.gl.TRIANGLES, brushTriangles(prop), color);
      this.drawArray(this.gl.LINES, brushLines(prop), outline);
    } else {
      this.drawArray(this.gl.TRIANGLES, cubeTriangles(center, scale, yaw), color);
      this.drawArray(this.gl.LINES, cubeLines(center, scale, yaw), outline);
    }
  }

  drawLevelSystems() {
    if (!state.level) {
      return;
    }
    const pathVertices = [];
    for (const path of state.level.paths || []) {
      const waypoints = (path.waypoints || []).map(vector);
      for (let index = 1; index < waypoints.length; index += 1) {
        pathVertices.push(...waypoints[index - 1], ...waypoints[index]);
      }
      for (const point of waypoints) {
        pathVertices.push(
          point[0] - 0.35, point[1], point[2],
          point[0] + 0.35, point[1], point[2],
          point[0], point[1], point[2] - 0.35,
          point[0], point[1], point[2] + 0.35,
          point[0], point[1] - 0.35, point[2],
          point[0], point[1] + 0.35, point[2]
        );
      }
    }

    const eventVertices = [];
    for (const event of state.level.events || []) {
      const trigger = event.trigger || {};
      const point = vector(trigger.position);
      const radius = Math.max(0.5, numberValue(trigger.radius) || 2.5);
      eventVertices.push(
        point[0] - radius, point[1], point[2],
        point[0] + radius, point[1], point[2],
        point[0], point[1], point[2] - radius,
        point[0], point[1], point[2] + radius,
        point[0], point[1] - 0.5, point[2],
        point[0], point[1] + 2.5, point[2]
      );
    }

    if (pathVertices.length === 0 && eventVertices.length === 0) {
      return;
    }
    const gl = this.gl;
    gl.disable(gl.DEPTH_TEST);
    this.drawArray(gl.LINES, pathVertices, [0.94, 0.75, 0.3, 1]);
    this.drawArray(gl.LINES, eventVertices, [0.41, 0.9, 0.82, 1]);
    gl.enable(gl.DEPTH_TEST);
  }

  drawBrushPreview() {
    if (!state.level || !state.drawBrush.start || !state.drawBrush.current) {
      return;
    }
    const prop = drawBrushProp(state.drawBrush.start, state.drawBrush.current);
    if (!prop) {
      return;
    }
    if (prop.brush_geometry) {
      this.drawArray(this.gl.TRIANGLES, brushTriangles(prop), [0.4, 1, 0.9, 0.30]);
      this.drawArray(this.gl.LINES, brushLines(prop), [0.8, 1, 0.95, 1]);
    } else {
      const center = vector(prop.position);
      const scale = vector(prop.scale);
      this.drawArray(this.gl.TRIANGLES, cubeTriangles(center, scale, 0), [0.4, 1, 0.9, 0.30]);
      this.drawArray(this.gl.LINES, cubeLines(center, scale, 0), [0.8, 1, 0.95, 1]);
    }
  }

  drawArray(mode, vertices, color) {
    if (vertices.length === 0) {
      return;
    }
    const gl = this.gl;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(vertices), gl.STREAM_DRAW);
    gl.enableVertexAttribArray(this.positionLocation);
    gl.vertexAttribPointer(this.positionLocation, 3, gl.FLOAT, false, 0, 0);
    gl.uniform4fv(this.colorLocation, new Float32Array(color));
    gl.drawArrays(mode, 0, vertices.length / 3);
  }
}

function makeProgram(gl, vertexSource, fragmentSource) {
  const vertex = compileShader(gl, gl.VERTEX_SHADER, vertexSource);
  const fragment = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSource);
  const program = gl.createProgram();
  gl.attachShader(program, vertex);
  gl.attachShader(program, fragment);
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    throw new Error(gl.getProgramInfoLog(program) || "failed to link WebGL program");
  }
  return program;
}

function compileShader(gl, type, source) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    throw new Error(gl.getShaderInfoLog(shader) || "failed to compile WebGL shader");
  }
  return shader;
}

function viewMatrix() {
  const eye = state.camera.position;
  const basis = cameraBasis();
  return [
    basis.right[0], basis.up[0], -basis.forward[0], 0,
    basis.right[1], basis.up[1], -basis.forward[1], 0,
    basis.right[2], basis.up[2], -basis.forward[2], 0,
    -dot3(basis.right, eye), -dot3(basis.up, eye), dot3(basis.forward, eye), 1,
  ];
}

function perspectiveMatrix(fov, aspect, near, far) {
  const f = 1 / Math.tan(fov * 0.5);
  return [
    f / aspect, 0, 0, 0,
    0, f, 0, 0,
    0, 0, (far + near) / (near - far), -1,
    0, 0, (2 * far * near) / (near - far), 0,
  ];
}

function multiplyMat4(a, b) {
  const out = new Array(16).fill(0);
  for (let col = 0; col < 4; col += 1) {
    for (let row = 0; row < 4; row += 1) {
      out[col * 4 + row] =
        a[0 * 4 + row] * b[col * 4 + 0] +
        a[1 * 4 + row] * b[col * 4 + 1] +
        a[2 * 4 + row] * b[col * 4 + 2] +
        a[3 * 4 + row] * b[col * 4 + 3];
    }
  }
  return out;
}

function cubeCorners(center, scale, yaw) {
  const half = scale.map((value) => Math.max(0.1, Math.abs(value) * 0.5));
  const cos = Math.cos(yaw);
  const sin = Math.sin(yaw);
  const corners = [];
  for (const x of [-half[0], half[0]]) {
    for (const y of [-half[1], half[1]]) {
      for (const z of [-half[2], half[2]]) {
        const rx = x * cos - z * sin;
        const rz = x * sin + z * cos;
        corners.push([center[0] + rx, center[1] + y, center[2] + rz]);
      }
    }
  }
  return corners;
}

function cubeTriangles(center, scale, yaw) {
  const c = cubeCorners(center, scale, yaw);
  const faces = [
    [0, 1, 3, 0, 3, 2],
    [4, 6, 7, 4, 7, 5],
    [0, 4, 5, 0, 5, 1],
    [2, 3, 7, 2, 7, 6],
    [0, 2, 6, 0, 6, 4],
    [1, 5, 7, 1, 7, 3],
  ];
  return faces.flatMap((face) => face.flatMap((index) => c[index]));
}

function cubeLines(center, scale, yaw) {
  const c = cubeCorners(center, scale, yaw);
  const edges = [
    [0, 1], [0, 2], [1, 3], [2, 3],
    [4, 5], [4, 6], [5, 7], [6, 7],
    [0, 4], [1, 5], [2, 6], [3, 7],
  ];
  return edges.flatMap(([a, b]) => [...c[a], ...c[b]]);
}

function colorToVec(hex, alpha) {
  const value = hex.replace("#", "");
  return [
    parseInt(value.slice(0, 2), 16) / 255,
    parseInt(value.slice(2, 4), 16) / 255,
    parseInt(value.slice(4, 6), 16) / 255,
    alpha,
  ];
}

function hexToRgba(hex, alpha) {
  const value = hex.replace("#", "");
  const r = parseInt(value.slice(0, 2), 16);
  const g = parseInt(value.slice(2, 4), 16);
  const b = parseInt(value.slice(4, 6), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function add3(a, b) {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
}

function sub3(a, b) {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

function scale3(v, scalar) {
  return [v[0] * scalar, v[1] * scalar, v[2] * scalar];
}

function dot3(a, b) {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

function cross3(a, b) {
  return [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ];
}

function length3(v) {
  return Math.hypot(v[0], v[1], v[2]);
}

function normalize3(v) {
  const length = length3(v);
  if (length <= 0.000001) {
    return [0, 0, 0];
  }
  return [v[0] / length, v[1] / length, v[2] / length];
}

function formatNumber(value) {
  return Number(value).toFixed(1).replace(/\.0$/, "");
}

function propLabel(prop, index) {
  if (prop.id) return `${index + 1} ${prop.id}`;
  if (prop.brush_geometry) return `${index + 1} Brush`;
  if (prop.enemy_type) return `${index + 1} Enemy ${prop.enemy_type}`;
  if (prop.item_id) return `${index + 1} Relic ${prop.item_id}`;
  if (prop.resource_value) return `${index + 1} Resource ${prop.resource_value}`;
  if (prop.anchor_id) return `${index + 1} Anchor ${prop.anchor_id}`;
  if (prop.trigger_level_id) return `${index + 1} Gate ${prop.trigger_level_id}`;
  if (prop.is_hurtbox) return `${index + 1} Hazard`;
  return `${index + 1} ${prop.asset_id || "Prop"}`;
}

function kindForProp(prop) {
  if (prop.enemy_type) return "enemy";
  if (prop.item_id || prop.resource_value) return "item";
  if (prop.anchor_id || prop.trigger_level_id || prop.is_hurtbox) return "entity";
  return "geometry";
}

function colorForKind(kind) {
  return {
    enemy: "#f36c5f",
    item: "#ffd66b",
    entity: "#9ee38f",
    geometry: "#78a9ff",
  }[kind];
}

function projectAssetCatalog() {
  const catalog = state.project?.asset_catalog || {};
  const runtimeModels = state.project?.assets || [];
  return {
    assets: catalog.assets || runtimeModels,
    models: catalog.models || runtimeModels,
    textures: catalog.textures || [],
    audio: catalog.audio || [],
    materials: catalog.materials || [],
    dialogue: catalog.dialogue || [],
    data: catalog.data || [],
    levels: catalog.levels || [],
    config: catalog.config || [],
  };
}

function assetOptions(formats, includeBaseMaps) {
  const assets = projectAssetCatalog().models.filter((asset) => asset.runtime_supported !== false);
  return assets
    .filter((asset) => includeBaseMaps || asset.root_path === "assets")
    .filter((asset) => formats.includes(asset.format))
    .map((asset) => (includeBaseMaps ? asset.full_path : asset.relative_path));
}

function uniqueImportId(base) {
  const used = new Set((state.level?.asset_imports || []).map((asset) => asset.id));
  base = base || "asset";
  let id = base;
  let index = 2;
  while (used.has(id)) {
    id = `${base}_${index}`;
    index += 1;
  }
  return id;
}

function slugFromPath(value) {
  const stem = String(value || "asset")
    .replace(/\\/g, "/")
    .split("/")
    .pop()
    .replace(/\.[^.]+$/, "");
  const slug = stem.toLowerCase().replace(/[^a-z0-9_-]+/g, "_").replace(/^_+|_+$/g, "");
  return slug || "asset";
}

function enemyIds() {
  return (state.project?.enemies || []).map((enemy) => enemy.id);
}

function relicIds() {
  return (state.project?.relics || []).map((relic) => relic.id);
}

function pickupAssetForRelic(id) {
  if (id === "veil_cinder") return "pickups/relic_veil_cinder.obj";
  if (id === "chain_sigil") return "pickups/relic_chain_sigil.obj";
  return "pickups/relic_ash_splinter.obj";
}

function enemyScale(id) {
  if (id === "burdened") return [1.5, 1.5, 1.5];
  return [1.2, 1.2, 1.2];
}

function defaultPlacementY() {
  return (state.level?.player_spawn?.[1] ?? 128) - 2;
}

function resetBrushWorkPlanes() {
  const spawn = vector(state.level?.player_spawn || [0, 128, 0]);
  state.drawBrush.groundY = defaultPlacementY();
  state.drawBrush.frontZ = spawn[2];
  state.drawBrush.sideX = spawn[0];
  el.brushGround.value = state.drawBrush.groundY;
  el.brushFrontZ.value = state.drawBrush.frontZ;
  el.brushSideX.value = state.drawBrush.sideX;
}

function vector(value) {
  const source = Array.isArray(value) ? value : [0, 0, 0];
  return [numberValue(source[0]), numberValue(source[1]), numberValue(source[2])];
}

function numberValue(value) {
  const number = Number(value);
  return Number.isFinite(number) ? number : 0;
}

function nullable(value) {
  const trimmed = String(value ?? "").trim();
  return trimmed.length ? trimmed : null;
}

function snap(value) {
  return state.transform.snap ? gridSnap(value) : numberValue(value);
}

function gridSnap(value) {
  const grid = Math.max(0.1, Number(state.gridSize) || 1);
  return Math.round(value / grid) * grid;
}

function sanitizeLevelId(value) {
  return String(value ?? "")
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}

function lerp(start, end, amount) {
  return start + (end - start) * amount;
}

function smoothStep(edge0, edge1, value) {
  if (edge0 === edge1) {
    return value < edge0 ? 0 : 1;
  }
  const amount = clamp((value - edge0) / (edge1 - edge0), 0, 1);
  return amount * amount * (3 - 2 * amount);
}

function titleCase(value) {
  return value.replace(/\b\w/g, (char) => char.toUpperCase());
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

installEvents();
initRenderer();
setViewLayout("quad");
setTool("select");
renderKeybindings();
renderAvailabilityControls();
requestAnimationFrame(animationFrame);
refreshProject();

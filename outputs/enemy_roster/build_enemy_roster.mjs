import fs from "node:fs/promises";
import path from "node:path";
import { SpreadsheetFile, Workbook } from "@oai/artifact-tool";

const repoRoot = path.resolve("../..");
const outputDir = path.resolve(".");
const enemiesDir = path.join(repoRoot, "data", "enemies");

function parseTomlValue(raw) {
  const value = raw.trim();
  if (value.startsWith('"') && value.endsWith('"')) {
    return value.slice(1, -1);
  }
  const numeric = Number(value);
  if (Number.isFinite(numeric)) {
    return numeric;
  }
  return value;
}

function parseSimpleToml(text) {
  const parsed = {};
  for (const line of text.split(/\r?\n/)) {
    const clean = line.split("#")[0].trim();
    if (!clean || clean.startsWith("[")) {
      continue;
    }
    const eq = clean.indexOf("=");
    if (eq < 0) {
      continue;
    }
    const key = clean.slice(0, eq).trim();
    const value = clean.slice(eq + 1);
    parsed[key] = parseTomlValue(value);
  }
  return parsed;
}

const files = (await fs.readdir(enemiesDir))
  .filter((file) => file.endsWith(".toml"))
  .sort();

const enemyDefinitions = [];
for (const file of files) {
  const fullPath = path.join(enemiesDir, file);
  const data = parseSimpleToml(await fs.readFile(fullPath, "utf8"));
  enemyDefinitions.push({
    sourceFile: `data/enemies/${file}`,
    ...data,
  });
}

const rosterRows = [
  ["Ashbound", "Grunt", "Low", "Low", "Medium", "Low", 1, "Implemented", "First Ash-Walk set", "Baseline melee pressure; proves core movement, aim, and spacing."],
  ["Burdened", "Tank", "High", "Medium", "Slow", "Medium", 2, "Implemented", "First Ash-Walk set", "Slow durable route blocker; should force repositioning."],
  ["Censer", "Glass cannon", "Low", "High", "Slow", "High", 3, "Implemented", "First Ash-Walk set", "Fragile priority target with readable wind-up."],
  ["Chainrunner", "High-speed flanker", "Low-Med", "Medium", "High", "High", 4, "Implemented data only", "Later Ash-Walk stress test", "Fast panic enemy; tests dash, camera, and audio tells."],
  ["Harpy", "Aerial", "Medium", "Medium", "High", "High", 5, "Implemented data only", "First Ash-Walk set", "Vertical threat; should punish exposed climbs."],
  ["Bellworn", "Ranged harasser", "Medium", "Low-Med", "Medium", "Medium", 8, "Planned", "Future", "Persistent ranged chip pressure and cover teaching."],
  ["Silencer", "Suppressor", "Low-Med", "Low", "Slow", "High", 7, "Planned", "Future", "Weakens or disables player tools after those tools are stable."],
  ["Paranoiac", "Escalator", "Low-Med", "Low", "Medium", "Very High", 6, "Planned", "Future", "Alarm enemy that worsens fights over time."],
  ["Anchor Parasite", "Swarmer", "Very Low", "Low", "High", "Low-Med", 9, "Planned", "Future", "Small pressure enemy around Anchors and machinery."],
  ["Root-Machine Hybrid", "Controller", "Med-High", "Medium", "Slow", "High", 10, "Planned", "Future", "Zone denial and arena control."],
  ["Mirror of the Player", "Duelist", "Medium", "High", "High", "Very High", 11, "Planned", "Late game", "Mimic/skill-check enemy for Mirror-Crust or special events."],
  ["Bell-Headed", "Elite modifier", "Varies", "Varies", "Varies", "Varies", 12, "Planned", "Modifier", "Elite frame applied to another base role."],
];

const workbook = Workbook.create();
const summary = workbook.worksheets.add("Summary");
const current = workbook.worksheets.add("Current Definitions");
const roster = workbook.worksheets.add("Design Roster");
const sources = workbook.worksheets.add("Sources");

for (const sheet of [summary, current, roster, sources]) {
  sheet.showGridLines = false;
}

const palette = {
  dark: "#1F2933",
  mid: "#536878",
  pale: "#E8ECEF",
  green: "#2F855A",
  amber: "#B7791F",
  red: "#B83232",
  blue: "#2B6CB0",
  text: "#111827",
  muted: "#6B7280",
};

function title(sheet, range, text) {
  const r = sheet.getRange(range);
  r.merge();
  r.values = [[text]];
  r.format = {
    fill: palette.dark,
    font: { bold: true, color: "#FFFFFF", size: 16 },
    horizontalAlignment: "left",
    verticalAlignment: "middle",
  };
  r.format.rowHeight = 30;
}

function styleHeader(range) {
  range.format = {
    fill: palette.mid,
    font: { bold: true, color: "#FFFFFF" },
    wrapText: true,
    horizontalAlignment: "center",
    verticalAlignment: "middle",
    borders: { preset: "outside", style: "thin", color: "#9AA6B2" },
  };
  range.format.rowHeight = 36;
}

function styleBody(range) {
  range.format = {
    font: { color: palette.text },
    wrapText: true,
    verticalAlignment: "top",
    borders: {
      insideHorizontal: { style: "thin", color: "#D8DEE6" },
      top: { style: "thin", color: "#C5CED8" },
      bottom: { style: "thin", color: "#C5CED8" },
    },
  };
}

// Current definitions
title(current, "A1:P1", "Current Enemy Definitions");
current.getRange("A3:P3").values = [[
  "Enemy ID",
  "Display Name",
  "Role",
  "Behavior Tag",
  "Model Asset",
  "Collider",
  "Visual Tell",
  "Health",
  "Damage",
  "Move Speed",
  "Activation Range",
  "Attack Range",
  "Attack Windup",
  "Attack Cooldown",
  "Implementation Status",
  "Source File",
]];
styleHeader(current.getRange("A3:P3"));

const currentRows = enemyDefinitions.map((enemy) => [
  enemy.id,
  enemy.display_name,
  enemy.role,
  enemy.behavior_tag,
  enemy.model_asset,
  enemy.collider_type,
  enemy.visual_tell,
  enemy.health,
  enemy.damage,
  enemy.move_speed,
  enemy.activation_range,
  enemy.attack_range,
  enemy.attack_windup,
  enemy.attack_cooldown,
  ["ashbound", "burdened", "censer", "chainrunner", "harpy"].includes(enemy.id) ? "Data + placeholder model" : "Planned",
  enemy.sourceFile,
]);
current.getRangeByIndexes(3, 0, currentRows.length, 16).values = currentRows;
styleBody(current.getRangeByIndexes(3, 0, currentRows.length, 16));
current.getRange("H4:N8").format.numberFormat = "#,##0.00";
current.getRange("A1:P1").format.borders = { preset: "outside", style: "medium", color: palette.dark };
current.freezePanes.freezeRows(3);
current.getRange("A:A").format.columnWidth = 16;
current.getRange("B:B").format.columnWidth = 18;
current.getRange("C:D").format.columnWidth = 18;
current.getRange("E:E").format.columnWidth = 28;
current.getRange("F:F").format.columnWidth = 12;
current.getRange("G:G").format.columnWidth = 44;
current.getRange("H:N").format.columnWidth = 14;
current.getRange("O:P").format.columnWidth = 24;

// Design roster
title(roster, "A1:J1", "Full Gameplay Enemy Roster");
roster.getRange("A3:J3").values = [[
  "Enemy",
  "Gameplay Role",
  "Health Band",
  "Damage Band",
  "Speed Band",
  "Priority",
  "Implementation Order",
  "Status",
  "Slice Use",
  "Design Note",
]];
styleHeader(roster.getRange("A3:J3"));
roster.getRangeByIndexes(3, 0, rosterRows.length, 10).values = rosterRows;
styleBody(roster.getRangeByIndexes(3, 0, rosterRows.length, 10));
roster.getRange("G4:G15").format.numberFormat = "0";
roster.freezePanes.freezeRows(3);
roster.getRange("A:B").format.columnWidth = 22;
roster.getRange("C:F").format.columnWidth = 14;
roster.getRange("G:G").format.columnWidth = 16;
roster.getRange("H:I").format.columnWidth = 22;
roster.getRange("J:J").format.columnWidth = 52;

// Summary
title(summary, "A1:H1", "Enemy Roster Summary");
summary.getRange("A3:B9").values = [
  ["Current data definitions", null],
  ["Planned roster entries", null],
  ["Average health", null],
  ["Average damage", null],
  ["Average move speed", null],
  ["Highest damage enemy", null],
  ["Fastest enemy", null],
];
summary.getRange("B3:B7").formulas = [
  ["=COUNTA('Current Definitions'!A4:A8)"],
  ["=COUNTA('Design Roster'!A4:A15)"],
  ["=AVERAGE('Current Definitions'!H4:H8)"],
  ["=AVERAGE('Current Definitions'!I4:I8)"],
  ["=AVERAGE('Current Definitions'!J4:J8)"],
];
summary.getRange("B8").formulas = [["=INDEX('Current Definitions'!B4:B8,MATCH(MAX('Current Definitions'!I4:I8),'Current Definitions'!I4:I8,0))"]];
summary.getRange("B9").formulas = [["=INDEX('Current Definitions'!B4:B8,MATCH(MAX('Current Definitions'!J4:J8),'Current Definitions'!J4:J8,0))"]];
summary.getRange("A3:A9").format = {
  fill: palette.pale,
  font: { bold: true, color: palette.text },
  borders: { preset: "outside", style: "thin", color: "#C5CED8" },
};
summary.getRange("B3:B9").format = {
  fill: "#FFFFFF",
  font: { bold: true, color: palette.blue },
  borders: { preset: "outside", style: "thin", color: "#C5CED8" },
};
summary.getRange("B5:B7").format.numberFormat = "#,##0.0";
summary.getRange("A11:H11").values = [["Implementation View", "Implemented / Data", "Planned", "Primary Ash-Walk", "Model Reset Priority", "Next Design Question", "Source", "Notes"]];
styleHeader(summary.getRange("A11:H11"));
summary.getRange("A12:H16").values = [
  ["Stage 0 Foundation", "Ashbound, Burdened, Censer, Chainrunner, Harpy", "Bellworn, Silencer, Paranoiac, Anchor Parasite, Root-Machine Hybrid, Mirror, Bell-Headed", "Ashbound, Burdened, Censer, Harpy", "Burdened then Ashbound", "Does movement, shooting, taking damage, and defeating an enemy feel good?", "ROADMAP.md", "Use foundation_test for smoke checks."],
  ["First Ascent", "Burdened prop, Anchor/resource stub", "Route choice, hazard, loot/relic stub", "Ashbound or Burdened baseline", "Anchor/resource/hazard props before enemy production art", "Is it fun to climb, survive, bank progress, and try again?", "MODEL_RESET_PLAN.md", "Keep models readable in gray first."],
  ["Threat Depth", "Baseline chase/attack only", "Role-specific AI, elite modifiers, suppression, aerial control", "Ashbound, Burdened, Censer, Harpy", "Enemy animation/audio tells", "Do enemies attack options, not only health?", "ENEMY_GAMEPLAY_ROSTER.txt", "Do not add suppression too early."],
  ["Model Reset", "Low-poly silhouettes", "Production models", "Burdened, Ashbound, Censer, Harpy", "Blockout kit first", "Can new assets replace cubes without breaking validation?", "ASSET_GUIDE.md", "Future-stable asset IDs matter."],
  ["Content Validation", "Enemy/model/config/level validation", "Relic/hazard registries", "N/A", "All referenced assets", "Can content expand without destabilizing core?", "FOUNDATION.md", "Run foundation_check before treating changes as stable."],
];
styleBody(summary.getRange("A12:H16"));
summary.getRange("A:A").format.columnWidth = 22;
summary.getRange("B:E").format.columnWidth = 28;
summary.getRange("F:F").format.columnWidth = 38;
summary.getRange("G:G").format.columnWidth = 26;
summary.getRange("H:H").format.columnWidth = 36;

// Source notes
title(sources, "A1:D1", "Workbook Sources");
sources.getRange("A3:D3").values = [["Source", "Type", "Used For", "Path"]];
styleHeader(sources.getRange("A3:D3"));
sources.getRange("A4:D12").values = [
  ["Enemy definition TOML files", "Project data", "Current implemented enemy stats and model references", "data/enemies/*.toml"],
  ["Enemy gameplay roster", "Design doc", "Full planned roster, roles, implementation order, and encounter intent", "ENEMY_GAMEPLAY_ROSTER.txt"],
  ["Enemy model generator brief", "Design doc", "Model readability and silhouette direction", "ENEMY_MODEL_GENERATOR_BRIEF.txt"],
  ["Foundation notes", "Technical doc", "Current runtime, validation, and smoke-test scope", "FOUNDATION.md"],
  ["Roadmap", "Planning doc", "Stage 0 and Stage 1 milestone context", "ROADMAP.md"],
  ["Asset guide", "Pipeline doc", "Model reset scale, naming, and replacement rules", "ASSET_GUIDE.md"],
  ["Model reset plan", "Pipeline doc", "Recommended asset rebuild sequence", "MODEL_RESET_PLAN.md"],
  ["Foundation test level", "Level data", "Current enemy/resource/Anchor test arena", "levels/foundation_test.json"],
  ["Current date", "Generation metadata", "Workbook generated locally from repo contents", "2026-07-08"],
];
styleBody(sources.getRange("A4:D12"));
sources.freezePanes.freezeRows(3);
sources.getRange("A:D").format.columnWidth = 34;

// Visual verification renders
for (const sheetName of ["Summary", "Current Definitions", "Design Roster", "Sources"]) {
  const preview = await workbook.render({ sheetName, autoCrop: "all", scale: 1, format: "png" });
  await fs.writeFile(
    path.join(outputDir, `${sheetName.toLowerCase().replaceAll(" ", "_")}.png`),
    new Uint8Array(await preview.arrayBuffer()),
  );
}

const inspect = await workbook.inspect({
  kind: "table",
  range: "Summary!A1:H16",
  include: "values,formulas",
  tableMaxRows: 20,
  tableMaxCols: 10,
});
console.log(inspect.ndjson);

const errors = await workbook.inspect({
  kind: "match",
  searchTerm: "#REF!|#DIV/0!|#VALUE!|#NAME\\?|#N/A",
  options: { useRegex: true, maxResults: 300 },
  summary: "final formula error scan",
});
console.log(errors.ndjson);

const xlsx = await SpreadsheetFile.exportXlsx(workbook);
await xlsx.save(path.join(outputDir, "cenotaph_enemy_roster.xlsx"));

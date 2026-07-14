"use strict";

(function installPrefabTools(global) {
  function clone(value) {
    return JSON.parse(JSON.stringify(value));
  }

  function vector3(value) {
    return [0, 1, 2].map((index) => {
      const number = Number(value?.[index]);
      return Number.isFinite(number) ? number : 0;
    });
  }

  function slug(value, fallback) {
    const normalized = String(value || "")
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9_-]+/g, "_")
      .replace(/^_+|_+$/g, "");
    return normalized || fallback;
  }

  function uniqueLocalId(value, fallback, used) {
    const base = slug(value, fallback);
    let candidate = base;
    let suffix = 2;
    while (used.has(candidate)) {
      candidate = `${base}_${suffix}`;
      suffix += 1;
    }
    used.add(candidate);
    return candidate;
  }

  function fromSelection(name, props, origin) {
    if (!Array.isArray(props) || props.length === 0) {
      throw new Error("Select at least one prop before creating a prefab.");
    }
    const pivot = vector3(origin);
    const propIds = new Set();
    const anchorIds = new Set();
    const relativeProps = props.map((source, index) => {
      const prop = clone(source);
      const position = vector3(prop.position);
      prop.position = position.map((value, axis) => value - pivot[axis]);
      prop.id = uniqueLocalId(
        prop.id || prop.asset_id,
        `prop_${index + 1}`,
        propIds
      );
      if (prop.anchor_id) {
        prop.anchor_id = uniqueLocalId(prop.anchor_id, `anchor_${index + 1}`, anchorIds);
      }
      return prop;
    });
    return {
      version: 1,
      name: String(name || "").trim(),
      props: relativeProps,
    };
  }

  function instantiate(prefab, origin) {
    if (!prefab || prefab.version !== 1 || !Array.isArray(prefab.props) || prefab.props.length === 0) {
      throw new Error("The selected prefab is malformed or uses an unsupported version.");
    }
    const target = vector3(origin);
    return prefab.props.map((source) => {
      const prop = clone(source);
      const position = vector3(prop.position);
      prop.position = position.map((value, axis) => value + target[axis]);
      return prop;
    });
  }

  global.CenotaphPrefabTools = Object.freeze({
    fromSelection,
    instantiate,
  });
})(window);

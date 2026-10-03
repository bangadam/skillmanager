import assert from "node:assert/strict";
import { test } from "node:test";
import { filterAndSortSkills } from "../src/views/sort-skills.ts";

const skills = [
  { id: "b", name: "Beta", description: "navigation tools", date_millis: 200 },
  { id: "a", name: "alpha", description: "Code review", date_millis: 100 },
  { id: "c", name: "Gamma", description: "NAVIGATION helpers", date_millis: null },
];
const ids = (items: typeof skills) => items.map((skill) => skill.id);

test("search includes name and description without case sensitivity; clearing restores all", () => {
  assert.deepEqual(ids(filterAndSortSkills(skills, "ALPHA", "created-desc")), ["a"]);
  assert.deepEqual(ids(filterAndSortSkills(skills, "navigation", "created-desc")), ["b", "c"]);
  assert.deepEqual(ids(filterAndSortSkills(skills, "", "created-desc")), ["b", "a", "c"]);
  assert.deepEqual(ids(skills), ["b", "a", "c"], "sorting never mutates the scan result");
});

test("name and creation date sort in either direction with unavailable dates last", () => {
  assert.deepEqual(ids(filterAndSortSkills(skills, "", "name-asc")), ["a", "b", "c"]);
  assert.deepEqual(ids(filterAndSortSkills(skills, "", "name-desc")), ["c", "b", "a"]);
  assert.deepEqual(ids(filterAndSortSkills(skills, "", "created-asc")), ["a", "b", "c"]);
  assert.deepEqual(ids(filterAndSortSkills(skills, "", "created-desc")), ["b", "a", "c"]);
});

// Tests of docs/book-assets/site-language.js (node --test scripts/site/tests/).
"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const lang = require("../../../docs/book-assets/site-language.js");

const codes = ["es", "pt"];

function decide(o) {
    return lang.decide(Object.assign({ codes: codes, search: "" }, o));
}

test("translated page never redirects", () => {
    assert.deepEqual(decide({ current: "es", stored: "pt", wanted: ["pt"] }),
        { go: null, remember: null });
});

test("?lang=en stores English", () => {
    assert.deepEqual(decide({ current: "en", search: "?lang=en", stored: "es" }),
        { go: null, remember: "en" });
});

test("stored translation is kept", () => {
    assert.equal(decide({ current: "en", stored: "es", wanted: ["en-US"] }).go, "es");
});

test("stored English stays", () => {
    assert.equal(decide({ current: "en", stored: "en", wanted: ["es"] }).go, null);
});

test("invalid stored value is ignored", () => {
    assert.equal(decide({ current: "en", stored: "xx", wanted: ["pt-BR"] }).go, "pt");
});

test("first visit, English first", () => {
    assert.equal(decide({ current: "en", stored: null, wanted: ["en-GB", "es"] }).go, null);
});

test("first visit, no match", () => {
    assert.equal(decide({ current: "en", stored: null, wanted: ["ja"] }).go, null);
});

test("missing wanted", () => {
    assert.equal(decide({ current: "en", stored: null, wanted: undefined }).go, null);
});

test("the storage key is the guide's", () => {
    assert.equal(lang.STORE, "fauste-player-guide-language");
});

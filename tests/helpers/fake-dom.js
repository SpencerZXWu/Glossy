"use strict";

const ATTRIBUTE_SELECTOR = /^\[([a-zA-Z0-9-]+)\]$/;

function escapeText(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function datasetKey(attribute) {
  return attribute
    .replace(/^data-/, "")
    .replace(/-([a-z0-9])/g, (match, letter) => letter.toUpperCase());
}

function descendantsOf(node, out) {
  for (const child of node.childNodes) {
    out.push(child);
    descendantsOf(child, out);
  }
  return out;
}

class FakeNode {
  constructor(document, tagName) {
    this.ownerDocument = document;
    this.tagName = String(tagName).toUpperCase();
    this.childNodes = [];
    this.parentNode = null;
    this.attributes = {};
    this.listeners = {};
    this.dataset = {};
    this.className = "";
    this.placeholder = "";
    this.title = "";
    this.lang = "";
    this.type = "";
    this.textValue = null;
    this.htmlValue = null;
  }

  get firstChild() {
    return this.childNodes.length ? this.childNodes[0] : null;
  }

  get textContent() {
    if (this.textValue !== null) return this.textValue;
    return this.childNodes.map((child) => child.textContent).join("");
  }

  set textContent(value) {
    this.textValue = value === null || value === undefined ? null : String(value);
    this.htmlValue = null;
    this.childNodes = [];
  }

  get innerHTML() {
    if (this.htmlValue !== null) return this.htmlValue;
    return this.childNodes.map((child) => child.outerHTML).join("");
  }

  set innerHTML(value) {
    this.htmlValue = String(value);
    this.textValue = null;
    this.childNodes = [];
  }

  get outerHTML() {
    const name = this.tagName.toLowerCase();
    const classAttribute = this.className ? ` class="${escapeText(this.className)}"` : "";
    const extra = Object.keys(this.attributes)
      .map((key) => ` ${key}="${escapeText(this.attributes[key])}"`)
      .join("");
    let inner;
    if (this.htmlValue !== null) inner = this.htmlValue;
    else if (this.textValue !== null) inner = escapeText(this.textValue);
    else inner = this.childNodes.map((child) => child.outerHTML).join("");
    return `<${name}${classAttribute}${extra}>${inner}</${name}>`;
  }

  appendChild(child) {
    child.parentNode = this;
    this.textValue = null;
    this.childNodes.push(child);
    return child;
  }

  removeChild(child) {
    const index = this.childNodes.indexOf(child);
    if (index !== -1) this.childNodes.splice(index, 1);
    child.parentNode = null;
    return child;
  }

  setAttribute(name, value) {
    this.attributes[name] = String(value);
  }

  getAttribute(name) {
    return name in this.attributes ? this.attributes[name] : null;
  }

  addEventListener(type, handler) {
    if (!this.listeners[type]) this.listeners[type] = [];
    this.listeners[type].push(handler);
  }

  dispatch(type, event) {
    const handlers = this.listeners[type] || [];
    for (const handler of handlers) handler(event || { type });
    return handlers.length;
  }

  hasAttribute(attribute) {
    return Object.prototype.hasOwnProperty.call(this.dataset, datasetKey(attribute));
  }

  querySelectorAll(selector) {
    const match = ATTRIBUTE_SELECTOR.exec(selector.trim());
    if (!match) return [];
    const attribute = match[1];
    return descendantsOf(this, []).filter((node) => node.hasAttribute(attribute));
  }
}

class FakeDocument {
  constructor() {
    this.documentElement = new FakeNode(this, "html");
    this.body = new FakeNode(this, "body");
    this.documentElement.appendChild(this.body);
  }

  createElement(tagName) {
    return new FakeNode(this, tagName);
  }

  querySelectorAll(selector) {
    return this.documentElement.querySelectorAll(selector);
  }

  appendChild(child) {
    return this.body.appendChild(child);
  }
}

function findByClass(root, className) {
  return descendantsOf(root, []).find((node) => node.className === className) || null;
}

function classesOf(root) {
  return descendantsOf(root, []).map((node) => node.className);
}

module.exports = { FakeDocument, FakeNode, findByClass, classesOf, descendantsOf };

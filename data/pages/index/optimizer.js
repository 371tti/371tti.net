class SpeculationRulesManager {
    constructor() {
        this.enabled = HTMLScriptElement.supports?.("speculationrules") ?? false;
        this.rules = {
            prerender: [],
            prefetch: []
        };
        this.scriptEl = null;
    }

    init() {
        if (!this.enabled) return;

        if (!this.scriptEl) {
            this.scriptEl = document.createElement("script");
            this.scriptEl.type = "speculationrules";
            document.head.appendChild(this.scriptEl);
        }
        this.#update();
    }

    #update() {
        if (!this.enabled || !this.scriptEl) return;
        this.scriptEl.textContent = JSON.stringify(this.rules, null, 2);
    }

    addPrerenderRule(rule) {
        this.rules.prerender.push(rule);
        this.#update();
    }

    addPrefetchRule(rule) {
        this.rules.prefetch.push(rule);
        this.#update();
    }

    clearPrerender() {
        this.rules.prerender = [];
        this.#update();
    }

    clearPrefetch() {
        this.rules.prefetch = [];
        this.#update();
    }

    clearAll() {
        this.rules = { prerender: [], prefetch: [] };
        this.#update();
    }

    getRules() {
        return JSON.parse(JSON.stringify(this.rules));
    }
}

window.SpeculationRules = new SpeculationRulesManager();
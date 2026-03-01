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

class PreloadController {
    constructor() {
        this.enabled = HTMLScriptElement.supports?.('speculationrules') ?? false;
        this.level = 1;
    }

    init() {
        if (!this.enabled) {
            console.log('[Preload] speculationrules is not supported in this browser');
            return;
        }
        this.level = this.#loadLevel();
        window.SpeculationRules.init();
        this.apply();
    }

    setLevel(level) {
        const n = parseInt(level, 10);
        if (Number.isNaN(n) || n < 0 || n > 3) {
            console.log('Invalid preload level:', level, '(expected 0,1,2,3)');
            return;
        }
        this.level = n;
        localStorage.setItem('preloadLevel', String(n));
        this.apply();
        console.log(`[Preload] level set to ${n}`);
    }

    getLevel() {
        return this.level;
    }

    handleCommand(args) {
        const raw = args && args[0] ? String(args[0]) : '';
        if (!raw) {
            console.log('Usage: Config preload <0|1|2|3>');
            console.log('  0: off (no prerender/prefetch)');
            console.log('  1: moderate (same-origin, moderate)');
            console.log('  2: local origin all (same-origin, eager)');
            console.log('  3: everything (same-origin eager + prefetch https)');
            return;
        }
        this.setLevel(raw);
    }

    apply() {
        if (!this.enabled) return;
        window.SpeculationRules.clearAll();

        switch (this.level) {
            case 0:
                break;
            case 1:
                window.SpeculationRules.addPrerenderRule({
                    where: { href_matches: '/*' },
                    eagerness: 'moderate',
                });
                break;
            case 2:
                window.SpeculationRules.addPrerenderRule({
                    where: { href_matches: '/*' },
                    eagerness: 'eager',
                });
                break;
            case 3:
                window.SpeculationRules.addPrerenderRule({
                    where: { href_matches: '/*' },
                    eagerness: 'eager',
                });
                window.SpeculationRules.addPrefetchRule({
                    where: { href_matches: 'https://*' },
                    eagerness: 'moderate',
                });
                break;
        }
    }

    #loadLevel() {
        const saved = parseInt(localStorage.getItem('preloadLevel') || '1', 10);
        if (Number.isNaN(saved)) return 1;
        return Math.min(Math.max(saved, 0), 3);
    }
}

const preloadController = new PreloadController();
preloadController.init();

window.RustyDocPreload = {
    setLevel: (level) => preloadController.setLevel(level),
    getLevel: () => preloadController.getLevel(),
};

window.RustyDocCommands = window.RustyDocCommands || {
    _pending: [],
    register(cmd) { this._pending.push(cmd); },
    list() { return []; }
};

window.RustyDocCommands.register({
    cmd: 'Config preload',
    desc: 'Configure prerender/prefetch level (0:off, 1:moderate, 2:local, 3:all)',
    action: (args) => preloadController.handleCommand(args),
    getCandidates: (parts) => {
        if (!parts || parts.length < 2 || parts[0].toLowerCase() !== 'config' || parts[1].toLowerCase() !== 'preload') {
            return [];
        }
        const options = [
            { level: 0, label: 'Off (no prerender/prefetch)' },
            { level: 1, label: 'Moderate (same-origin, moderate)' },
            { level: 2, label: 'Local origin all (same-origin, eager)' },
            { level: 3, label: 'Everything (eager + prefetch https)' },
        ];
        const currentLevel = preloadController.getLevel();
        return options.map(opt => ({
            cmd: `Config preload ${opt.level}`,
            desc: `${opt.label} ${currentLevel === opt.level ? '(current)' : ''}`,
            action: () => preloadController.setLevel(opt.level),
        }));
    },
});
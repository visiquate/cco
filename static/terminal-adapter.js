/**
 * Terminal Adapter Pattern
 * Provides abstraction layer for switching between xterm.js and WASM terminal
 */

class TerminalAdapter {
    constructor(container) {
        this.container = container;
        this.wsUrl = null;
        this.ws = null;
        this.terminal = null;
        this.isConnected = false;

        // Event callbacks
        this.onConnect = null;
        this.onDisconnect = null;
        this.onError = null;
    }

    /**
     * Connect to WebSocket
     * @param {string} wsUrl
     * @returns {Promise<void>}
     */
    async connect(wsUrl) {
        this.wsUrl = wsUrl;

        return new Promise((resolve, reject) => {
            try {
                this.ws = new WebSocket(wsUrl);
                this.ws.binaryType = 'arraybuffer';

                this.ws.onopen = () => {
                    console.log('WebSocket connected');
                    this.isConnected = true;
                    if (this.onConnect) {
                        this.onConnect();
                    }
                    resolve();
                };

                this.ws.onmessage = (event) => {
                    this.handleMessage(event);
                };

                this.ws.onerror = (error) => {
                    console.error('WebSocket error:', error);
                    if (this.onError) {
                        this.onError(error);
                    }
                    reject(error);
                };

                this.ws.onclose = () => {
                    console.log('WebSocket disconnected');
                    this.isConnected = false;
                    if (this.onDisconnect) {
                        this.onDisconnect();
                    }
                    this.scheduleReconnect();
                };
            } catch (error) {
                reject(error);
            }
        });
    }

    /**
     * Handle WebSocket message
     * @param {MessageEvent} event
     */
    handleMessage(event) {
        // Override in subclass
    }

    /**
     * Send data to WebSocket
     * @param {string} data
     */
    send(data) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(data);
        }
    }

    /**
     * Close connection
     */
    close() {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
        if (this.terminal) {
            this.terminal.destroy();
            this.terminal = null;
        }
    }

    /**
     * Schedule reconnection
     */
    scheduleReconnect() {
        setTimeout(() => {
            if (!this.isConnected && this.wsUrl) {
                console.log('Attempting to reconnect...');
                this.connect(this.wsUrl).catch(error => {
                    console.error('Reconnection failed:', error);
                });
            }
        }, 3000);
    }

    /**
     * Handle resize
     * @param {number} cols
     * @param {number} rows
     */
    resize(cols, rows) {
        // Send resize command to server
        const resizeCmd = JSON.stringify({
            type: 'resize',
            cols: cols,
            rows: rows
        });
        this.send(resizeCmd);
    }
}

/**
 * WASM Terminal Adapter
 */
class WasmTerminalAdapter extends TerminalAdapter {
    constructor(container) {
        super(container);
        this.canvasId = 'wasm-terminal-canvas';
    }

    /**
     * Initialize terminal
     * @returns {Promise<void>}
     */
    async init() {
        try {
            // Create canvas element
            const canvas = document.createElement('canvas');
            canvas.id = this.canvasId;
            canvas.className = 'terminal-canvas';
            canvas.tabIndex = 0;
            this.container.appendChild(canvas);

            // Create WASM terminal
            this.terminal = new WasmTerminal(this.canvasId);

            // Setup callbacks
            this.terminal.onData = (data) => {
                this.send(data);
            };

            this.terminal.onResize = (cols, rows) => {
                this.resize(cols, rows);
            };

            this.terminal.onError = (error) => {
                console.error('WASM terminal error:', error);
                if (this.onError) {
                    this.onError(error);
                }
            };

            // Initialize WASM terminal
            await this.terminal.init();

            // Auto-resize to container
            this.terminal.autoResize();

            console.log('WASM terminal adapter initialized');
        } catch (error) {
            console.error('Failed to initialize WASM terminal adapter:', error);
            throw error;
        }
    }

    /**
     * Handle WebSocket message
     * @param {MessageEvent} event
     */
    handleMessage(event) {
        if (!this.terminal) return;

        let data;
        if (event.data instanceof ArrayBuffer) {
            // Convert ArrayBuffer to string
            const decoder = new TextDecoder();
            data = decoder.decode(new Uint8Array(event.data));
        } else {
            data = event.data;
        }

        // Write to terminal
        this.terminal.write(data);
    }

    /**
     * Clear terminal
     */
    clear() {
        if (this.terminal) {
            this.terminal.clear();
        }
    }

    /**
     * Focus terminal
     */
    focus() {
        const canvas = document.getElementById(this.canvasId);
        if (canvas) {
            canvas.focus();
        }
    }
}

/**
 * XTerm Adapter (fallback)
 */
class XtermAdapter extends TerminalAdapter {
    constructor(container) {
        super(container);
        this.fitAddon = null;
        this.webLinksAddon = null;
    }

    /**
     * Initialize terminal
     * @returns {Promise<void>}
     */
    async init() {
        try {
            // Check if xterm.js is available
            if (typeof Terminal === 'undefined') {
                throw new Error('xterm.js not loaded');
            }

            // Create xterm terminal
            this.terminal = new Terminal({
                cols: 80,
                rows: 24,
                fontFamily: 'Fira Code, Consolas, Monaco, monospace',
                fontSize: 14,
                theme: {
                    background: '#000000',
                    foreground: '#ffffff',
                    cursor: '#00ff00',
                    cursorAccent: '#00ff00'
                },
                cursorBlink: true,
                scrollback: 10000
            });

            // Load addons
            this.fitAddon = new FitAddon.FitAddon();
            this.terminal.loadAddon(this.fitAddon);

            try {
                this.webLinksAddon = new WebLinksAddon.WebLinksAddon();
                this.terminal.loadAddon(this.webLinksAddon);
            } catch (e) {
                console.warn('WebLinks addon not available:', e);
            }

            // Open terminal in container
            this.terminal.open(this.container);

            // Setup data handler
            this.terminal.onData((data) => {
                this.send(data);
            });

            // Setup resize handler
            this.terminal.onResize(({ cols, rows }) => {
                this.resize(cols, rows);
            });

            // Fit to container
            this.fitAddon.fit();

            // Auto-resize on window resize
            window.addEventListener('resize', () => {
                if (this.fitAddon) {
                    this.fitAddon.fit();
                }
            });

            console.log('xterm adapter initialized');
        } catch (error) {
            console.error('Failed to initialize xterm adapter:', error);
            throw error;
        }
    }

    /**
     * Handle WebSocket message
     * @param {MessageEvent} event
     */
    handleMessage(event) {
        if (!this.terminal) return;

        let data;
        if (event.data instanceof ArrayBuffer) {
            // Convert ArrayBuffer to string
            const decoder = new TextDecoder();
            data = decoder.decode(new Uint8Array(event.data));
        } else {
            data = event.data;
        }

        // Write to terminal
        this.terminal.write(data);
    }

    /**
     * Clear terminal
     */
    clear() {
        if (this.terminal) {
            this.terminal.clear();
        }
    }

    /**
     * Focus terminal
     */
    focus() {
        if (this.terminal) {
            this.terminal.focus();
        }
    }
}

/**
 * Terminal Manager - Factory for creating appropriate adapter
 */
class TerminalManager {
    /**
     * Create terminal adapter based on preference and availability
     * @param {string} container - Container element ID
     * @param {string} preference - 'wasm' or 'xterm'
     * @returns {TerminalAdapter}
     */
    static async create(container, preference = 'wasm') {
        const containerEl = typeof container === 'string'
            ? document.getElementById(container)
            : container;

        if (!containerEl) {
            throw new Error('Terminal container not found');
        }

        let adapter = null;

        // Try to create preferred adapter
        if (preference === 'wasm') {
            try {
                console.log('Attempting to create WASM terminal...');
                adapter = new WasmTerminalAdapter(containerEl);
                await adapter.init();
                return adapter;
            } catch (error) {
                console.warn('Failed to create WASM terminal, falling back to xterm:', error);
            }
        }

        // Fallback to xterm.js
        try {
            console.log('Creating xterm terminal...');
            adapter = new XtermAdapter(containerEl);
            await adapter.init();
            return adapter;
        } catch (error) {
            console.error('Failed to create xterm terminal:', error);
            throw new Error('No terminal implementation available');
        }
    }

    /**
     * Check if WASM terminal is supported
     * @returns {boolean}
     */
    static isWasmSupported() {
        // Check for WebAssembly support
        if (typeof WebAssembly === 'undefined') {
            return false;
        }

        // Check for required browser APIs
        if (!window.CanvasRenderingContext2D || !window.TextDecoder) {
            return false;
        }

        return true;
    }

    /**
     * Get terminal preference from localStorage
     * @returns {string}
     */
    static getPreference() {
        return localStorage.getItem('terminalType') || 'wasm';
    }

    /**
     * Save terminal preference to localStorage
     * @param {string} type
     */
    static savePreference(type) {
        localStorage.setItem('terminalType', type);
    }
}
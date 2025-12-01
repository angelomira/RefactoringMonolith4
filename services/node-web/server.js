/**
 * Space Dashboard - Node.js/Express Web Service
 * Migrated from PHP/Laravel
 * 
 * This service provides:
 * - Dashboard with ISS tracking and JWST gallery
 * - API proxy to rust_iss service
 * - JWST image feed with filtering
 * - Astronomy events from AstronomyAPI
 */

const express = require('express');
const axios = require('axios');
const { Pool } = require('pg');
const path = require('path');

const app = express();
const PORT = process.env.PORT || 3000;

// Configuration
const config = {
    rustBase: process.env.RUST_BASE || 'http://rust_iss:3000',
    jwstHost: process.env.JWST_HOST || 'https://api.jwstapi.com',
    jwstApiKey: process.env.JWST_API_KEY || '',
    astroAppId: process.env.ASTRO_APP_ID || '',
    astroAppSecret: process.env.ASTRO_APP_SECRET || '',
    db: {
        host: process.env.DB_HOST || 'db',
        port: parseInt(process.env.DB_PORT || '5432'),
        database: process.env.DB_DATABASE || 'monolith',
        user: process.env.DB_USERNAME || 'monouser',
        password: process.env.DB_PASSWORD || 'monopass'
    }
};

// Database connection pool
const pool = new Pool(config.db);

// View engine setup
app.set('view engine', 'ejs');
app.set('views', path.join(__dirname, 'views'));

// Static files
app.use(express.static(path.join(__dirname, 'public')));
app.use(express.json());

// HTTP client with timeout
const httpClient = axios.create({
    timeout: 5000,
    headers: {
        'User-Agent': 'node-web/1.0'
    }
});

/**
 * Helper: Fetch JSON from URL with error handling
 */
async function fetchJson(url, options = {}) {
    try {
        const response = await httpClient.get(url, options);
        return response.data;
    } catch (error) {
        console.error(`Error fetching ${url}:`, error.message);
        return {};
    }
}

/**
 * Helper: Pick valid image URL from JWST item
 */
function pickImageUrl(item) {
    const candidates = [
        item.location,
        item.url,
        item.thumbnail
    ];
    
    for (const url of candidates) {
        if (typeof url === 'string' && /\.(jpg|jpeg|png)(\?.*)?$/i.test(url)) {
            return url;
        }
    }
    return null;
}

// ==================== ROUTES ====================

/**
 * Home - redirect to dashboard
 */
app.get('/', (req, res) => {
    res.redirect('/dashboard');
});

/**
 * Dashboard - main page with ISS map, JWST gallery, astronomy events
 */
app.get('/dashboard', async (req, res) => {
    try {
        // Fetch ISS data from rust_iss
        const issData = await fetchJson(`${config.rustBase}/last`);
        const payload = issData.payload || {};
        
        res.render('dashboard', {
            iss: issData,
            metrics: {
                issSpeed: payload.velocity || null,
                issAlt: payload.altitude || null
            }
        });
    } catch (error) {
        console.error('Dashboard error:', error);
        res.render('dashboard', {
            iss: {},
            metrics: { issSpeed: null, issAlt: null }
        });
    }
});

/**
 * API: Proxy to rust_iss /last endpoint
 */
app.get('/api/iss/last', async (req, res) => {
    try {
        const data = await fetchJson(`${config.rustBase}/last`);
        res.json(data);
    } catch (error) {
        res.json({ error: 'upstream' });
    }
});

/**
 * API: Proxy to rust_iss /iss/trend endpoint
 */
app.get('/api/iss/trend', async (req, res) => {
    try {
        const queryString = new URLSearchParams(req.query).toString();
        const url = `${config.rustBase}/iss/trend${queryString ? '?' + queryString : ''}`;
        const data = await fetchJson(url);
        res.json(data);
    } catch (error) {
        res.json({ error: 'upstream' });
    }
});

/**
 * API: JWST image feed with filtering
 * Query params:
 *  - source: jpg|suffix|program (default: jpg)
 *  - suffix: e.g., _cal, _thumb
 *  - program: program ID
 *  - instrument: NIRCam|MIRI|NIRISS|NIRSpec|FGS
 *  - page, perPage
 */
app.get('/api/jwst/feed', async (req, res) => {
    try {
        const source = req.query.source || 'jpg';
        const suffix = (req.query.suffix || '').trim();
        const program = (req.query.program || '').trim();
        const instrument = (req.query.instrument || '').toUpperCase().trim();
        const page = Math.max(1, parseInt(req.query.page) || 1);
        const perPage = Math.max(1, Math.min(60, parseInt(req.query.perPage) || 24));

        // Build API path
        let apiPath = 'all/type/jpg';
        if (source === 'suffix' && suffix) {
            apiPath = `all/suffix/${suffix.replace(/^\//, '')}`;
        } else if (source === 'program' && program) {
            apiPath = `program/id/${encodeURIComponent(program)}`;
        }

        // Fetch from JWST API
        const url = `${config.jwstHost}/${apiPath}`;
        const response = await fetchJson(url, {
            params: { page, perPage },
            headers: config.jwstApiKey ? { 'X-API-Key': config.jwstApiKey } : {}
        });

        const list = response.body || response.data || (Array.isArray(response) ? response : []);
        const items = [];

        for (const item of list) {
            if (!item || typeof item !== 'object') continue;

            // Find valid image URL
            const imageUrl = pickImageUrl(item);
            if (!imageUrl) continue;

            // Extract instruments
            const instruments = [];
            if (item.details?.instruments) {
                for (const inst of item.details.instruments) {
                    if (inst?.instrument) {
                        instruments.push(inst.instrument.toUpperCase());
                    }
                }
            }

            // Filter by instrument if specified
            if (instrument && instruments.length && !instruments.includes(instrument)) {
                continue;
            }

            // Build caption
            const obsId = item.observation_id || item.observationId || item.id || '';
            const prog = item.program || '-';
            const sfx = item.details?.suffix || item.suffix || '';
            
            let caption = obsId;
            caption += ` · P${prog}`;
            if (sfx) caption += ` · ${sfx}`;
            if (instruments.length) caption += ` · ${instruments.join('/')}`;

            items.push({
                url: imageUrl,
                obs: String(item.observation_id || item.observationId || ''),
                program: String(item.program || ''),
                suffix: sfx,
                inst: instruments,
                caption: caption.trim(),
                link: item.location || imageUrl
            });

            if (items.length >= perPage) break;
        }

        res.json({
            source: apiPath,
            count: items.length,
            items
        });
    } catch (error) {
        console.error('JWST feed error:', error);
        res.json({ source: '', count: 0, items: [], error: error.message });
    }
});

/**
 * API: Astronomy events from AstronomyAPI
 */
app.get('/api/astro/events', async (req, res) => {
    try {
        const lat = parseFloat(req.query.lat) || 55.7558;
        const lon = parseFloat(req.query.lon) || 37.6176;
        const days = Math.min(30, Math.max(1, parseInt(req.query.days) || 7));

        if (!config.astroAppId || !config.astroAppSecret) {
            return res.json({ error: 'AstronomyAPI not configured', events: [] });
        }

        // Build date range
        const now = new Date();
        const end = new Date(now.getTime() + days * 24 * 60 * 60 * 1000);
        
        const authString = Buffer.from(`${config.astroAppId}:${config.astroAppSecret}`).toString('base64');
        
        const response = await httpClient.get('https://api.astronomyapi.com/api/v2/bodies/events', {
            headers: {
                'Authorization': `Basic ${authString}`
            },
            params: {
                latitude: lat,
                longitude: lon,
                from_date: now.toISOString().split('T')[0],
                to_date: end.toISOString().split('T')[0],
                elevation: 0,
                time: '00:00:00'
            }
        });

        res.json(response.data);
    } catch (error) {
        console.error('Astro events error:', error);
        res.json({ error: error.message, events: [] });
    }
});

/**
 * OSDR page - list of datasets
 */
app.get('/osdr', async (req, res) => {
    try {
        const data = await fetchJson(`${config.rustBase}/osdr/list`);
        res.render('osdr', { items: data.items || [] });
    } catch (error) {
        res.render('osdr', { items: [] });
    }
});

/**
 * CMS page by slug
 */
app.get('/page/:slug', async (req, res) => {
    try {
        const result = await pool.query(
            'SELECT title, body FROM cms_pages WHERE slug = $1 LIMIT 1',
            [req.params.slug]
        );
        
        if (result.rows.length === 0) {
            return res.status(404).render('page', { 
                title: 'Страница не найдена', 
                body: '<p>Запрашиваемая страница не существует.</p>' 
            });
        }
        
        res.render('page', result.rows[0]);
    } catch (error) {
        console.error('CMS page error:', error);
        res.status(500).render('page', { 
            title: 'Ошибка', 
            body: '<p>Ошибка загрузки страницы.</p>' 
        });
    }
});

/**
 * Health check endpoint
 */
app.get('/health', (req, res) => {
    res.json({ status: 'ok', service: 'node-web', timestamp: new Date().toISOString() });
});

// Error handling
app.use((err, req, res, next) => {
    console.error('Unhandled error:', err);
    res.status(500).json({ error: 'Internal server error' });
});

// Start server
app.listen(PORT, '0.0.0.0', () => {
    console.log(`✓ Node.js web service started on port ${PORT}`);
    console.log(`  Rust API: ${config.rustBase}`);
    console.log(`  JWST API: ${config.jwstHost}`);
    console.log(`  Database: ${config.db.host}:${config.db.port}/${config.db.database}`);
});

module.exports = app;

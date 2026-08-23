const { execSync, spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const schemaPath = path.resolve(__dirname, '../crates/lol_web_server/migrations/schema.sql');
const rlSchemaPath = path.resolve(__dirname, '../crates/lol_rl/migrations/schema.sql');

// 解析 DATABASE_URL 或环境变量
const rawDbUrl = process.env.DATABASE_URL || 'postgres://postgres:postgres@localhost:5432/moon_lol';
let dbConfig = {
    user: process.env.PGUSER || 'postgres',
    password: process.env.PGPASSWORD || 'postgres',
    host: process.env.PGHOST || 'localhost',
    port: process.env.PGPORT || '5432',
    database: process.env.PGDATABASE || 'moon_lol',
};

try {
    const parsed = new URL(rawDbUrl);
    if (parsed.username) dbConfig.user = parsed.username;
    if (parsed.password) dbConfig.password = parsed.password;
    if (parsed.hostname) dbConfig.host = parsed.hostname;
    if (parsed.port) dbConfig.port = parsed.port;
    if (parsed.pathname && parsed.pathname.length > 1) {
        dbConfig.database = parsed.pathname.slice(1);
    }
} catch (e) {
    // 使用默认或环境变量
}

// 检查本地是否有 psql
function findPsqlPath() {
    const testPath = spawnSync(process.platform === 'win32' ? 'where.exe' : 'which', ['psql'], { encoding: 'utf8' });
    if (testPath.status === 0 && testPath.stdout && testPath.stdout.trim()) {
        return testPath.stdout.trim().split(/\r?\n/)[0];
    }
    if (process.platform === 'win32') {
        const defaultWinPaths = [
            'C:\\Program Files\\PostgreSQL\\17\\bin\\psql.exe',
            'C:\\Program Files\\PostgreSQL\\16\\bin\\psql.exe',
            'C:\\Program Files\\PostgreSQL\\15\\bin\\psql.exe',
            'C:\\Program Files\\PostgreSQL\\14\\bin\\psql.exe',
        ];
        for (const p of defaultWinPaths) {
            if (fs.existsSync(p)) return p;
        }
    }
    return null;
}

const localPsql = findPsqlPath();

function runPsql(db, sqlOrCommand, isStdin = false) {
    const env = { ...process.env, PGPASSWORD: dbConfig.password };
    if (localPsql) {
        let args = ['-h', dbConfig.host, '-p', dbConfig.port, '-U', dbConfig.user, '-d', db];
        if (isStdin) {
            console.log(`执行: ${localPsql} ${args.join(' ')} (stdin)`);
            const res = spawnSync(localPsql, args, { env, input: sqlOrCommand, stdio: ['pipe', 'inherit', 'inherit'] });
            if (res.status !== 0) throw new Error(`psql failed with exit code ${res.status}`);
        } else {
            args.push('-c', sqlOrCommand);
            console.log(`执行: ${localPsql} ${args.join(' ')}`);
            const res = spawnSync(localPsql, args, { env, stdio: 'inherit' });
            if (res.status !== 0) throw new Error(`psql failed with exit code ${res.status}`);
        }
    } else {
        // 使用 Docker 容器
        console.log('未检测到本地 psql，尝试使用 Docker 容器 moon-lol-postgres...');
        if (isStdin) {
            execSync(`docker exec -i moon-lol-postgres psql -U ${dbConfig.user} -d ${db}`, {
                input: sqlOrCommand,
                stdio: ['pipe', 'inherit', 'inherit'],
            });
        } else {
            execSync(`docker exec -i moon-lol-postgres psql -U ${dbConfig.user} -d ${db} -c "${sqlOrCommand.replace(/"/g, '\\"')}"`, {
                stdio: 'inherit',
            });
        }
    }
}

try {
    console.log(`开始重置数据库 [${dbConfig.database}] (主机: ${dbConfig.host}:${dbConfig.port}, 用户: ${dbConfig.user})...`);

    // 1. 强制删除并重建数据库
    runPsql('postgres', `DROP DATABASE IF EXISTS ${dbConfig.database} WITH (FORCE);`);
    runPsql('postgres', `CREATE DATABASE ${dbConfig.database};`);

    // 2. 导入 Schema (web server)
    const schemaSql = fs.readFileSync(schemaPath, 'utf8');
    runPsql(dbConfig.database, schemaSql, true);

    // 2b. 导入 RL Schema
    if (fs.existsSync(rlSchemaPath)) {
        const rlSchemaSql = fs.readFileSync(rlSchemaPath, 'utf8');
        runPsql(dbConfig.database, rlSchemaSql, true);
    }

    // 3. 导入初始化套餐数据和默认赛季
    const seedSql = `
INSERT INTO billing_plans (id, name, price_cents, essence_per_month, max_agents, sort_order) VALUES
('free', '免费版', 0, 0, 5, 0),
('pro', '专业版', 2900, 3000, 20, 1),
('elite', '精英版', 9900, 12000, 100, 2)
ON CONFLICT (id) DO NOTHING;

INSERT INTO seasons (id, name, mode, starts_at, ends_at, status) VALUES
('d3b07384-d113-4c4e-9c8e-cf003cfb9fbe', 'S1 赛季', 'top_solo', NOW() - INTERVAL '1 day', NOW() + INTERVAL '30 days', 'active')
ON CONFLICT (id) DO NOTHING;
`;
    runPsql(dbConfig.database, seedSql, true);

    console.log(`\n🎉 数据库 ${dbConfig.database} 重置并初始化成功！`);
} catch (error) {
    console.error('❌ 数据库重置失败:', error.message);
    process.exit(1);
}

const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const schemaPath = path.resolve(__dirname, '../crates/lol_web_server/migrations/schema.sql');

function runCmd(cmd, stdin = null) {
    console.log(`执行: ${cmd}`);
    if (stdin) {
        execSync(cmd, { input: stdin, stdio: ['pipe', 'inherit', 'inherit'] });
    } else {
        execSync(cmd, { stdio: 'inherit' });
    }
}

try {
    console.log('开始重置数据库...');

    // 1. 强制删除并重建数据库
    runCmd('docker exec -i moon-lol-postgres psql -U postgres -d postgres -c "DROP DATABASE IF EXISTS moon_lol WITH (FORCE);"');
    runCmd('docker exec -i moon-lol-postgres psql -U postgres -d postgres -c "CREATE DATABASE moon_lol;"');

    // 2. 导入 Schema
    const schemaSql = fs.readFileSync(schemaPath, 'utf8');
    runCmd('docker exec -i moon-lol-postgres psql -U postgres -d moon_lol', schemaSql);

    // 3. 导入初始化套餐数据
    const seedSql = `
INSERT INTO billing_plans (id, name, price_cents, essence_per_month, max_agents, sort_order) VALUES
('free', '免费版', 0, 0, 5, 0),
('pro', '专业版', 2900, 3000, 20, 1),
('elite', '精英版', 9900, 12000, 100, 2)
ON CONFLICT (id) DO NOTHING;
`;
    runCmd('docker exec -i moon-lol-postgres psql -U postgres -d moon_lol', seedSql);

    console.log('数据库重置成功！');
} catch (error) {
    console.error('数据库重置失败:', error.message);
    process.exit(1);
}

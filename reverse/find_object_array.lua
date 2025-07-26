local ADDR_GAME_PLAYER = "League of Legends.exe + 1AEEFA0"
local ADDR_GAME_TIME = "League of Legends.exe + 01AD1F70"

local OFFSET = {
    INDEX = 0x10,
    TEAM = 0x249,        -- byte (1字节)
    POS = 0x1f0,         -- Vec3 (3个浮点数)
    NAME = 0x4098,       -- string (字符串)
    BASE_ATT = 0x179c,   -- float (浮点数)
    ATT_RANGE = 0x17e4,  -- float (浮点数)
    HEALTH = 0x1078,     -- float (浮点数)
    MAX_HEALTH = 0x10a0, -- float (浮点数)
    LEVEL = 0x4cf0,      -- float (浮点数)
    TARGETABLE = 0xea0,  -- byte (1字节)
    IS_ALIVE = 0x248,    -- byte (1字节)
    ARMOR = 0x17c4,      -- float (浮点数)
    MAG_RES = 0x17cc,    -- float (浮点数)
    MOVE_SPEED = 0x17dc, -- float (浮点数)
    BONUS_ATT = 0x16f8,  -- float (浮点数)
    AP = 0x1708,         -- float (浮点数)
    RADIUS = 0x6D0,      -- float (浮点数)
    SCALE = 0x17B4       -- float (浮点数)
}


-- 读取玩家index
local function read_player_index(player_addr)
    return readSmallInteger(player_addr + OFFSET.INDEX)
end

-- 验证数组地址是否有效
local function verify_array_addr(array_addr, index)
    local next_obj_addr = readQword(array_addr + 0x8 * (index + 1))
    if next_obj_addr == 0 then return false end

    local next_index = read_player_index(next_obj_addr)
    return next_index == index + 1
end

-- 主搜索函数
function find_object_array(player_addr)
    local player_index = read_player_index(player_addr)

    if player_index == nil or player_index == 0 then
        return nil
    end

    -- 搜索包含player_addr的指针
    -- 将8字节地址转换为字节序列
    local addr_bytes = string.format("%02X %02X %02X %02X %02X %02X %02X %02X",
        player_addr & 0xFF,
        (player_addr >> 8) & 0xFF,
        (player_addr >> 16) & 0xFF,
        (player_addr >> 24) & 0xFF,
        (player_addr >> 32) & 0xFF,
        (player_addr >> 40) & 0xFF,
        (player_addr >> 48) & 0xFF,
        (player_addr >> 56) & 0xFF
    )

    local array_addr_array = AOBScan(addr_bytes)
    
    -- 检查并打印 array_addr_array 的基本属性
    if array_addr_array then

        -- 遍历并打印每个搜索结果的信息
        for i=1, array_addr_array.Count do
            local array_addr_str = array_addr_array[i-1]  -- 注意：索引从0开始
            local array_addr = tonumber(array_addr_str, 16)
            local potential_array_addr = array_addr - (player_index * 8)

            -- 验证这个地址是否为真正的数组地址
            if verify_array_addr(potential_array_addr, player_index) then
                array_addr_array.destroy()
                return potential_array_addr
            end
        end
        
        -- 使用完后记得销毁
        array_addr_array.destroy()
    else
        print("未找到结果")
    end

    return nil
end


-- 获取对象列表的基址
local ADDR_OBJ_LIST = find_object_array(readQword(ADDR_GAME_PLAYER))

-- 安全读取函数
local function safeRead(readFunc, address, ...)
    local success, result = pcall(readFunc, address, ...)
    return success and result or 0
end

-- 获取单个对象信息的函数
local function getObjectInfo(objAddress, index)
    if objAddress == nil or objAddress == 0 then
        return nil
    end

    -- 读取对象名称
    local namePtr = readQword(objAddress + OFFSET.NAME)
    local name = ""
    if namePtr ~= nil and namePtr ~= 0 then
        name = readString(namePtr, 64)
    end
    
    return {
        index = index,
        name = name or "未知",
        team = safeRead(readBytes, objAddress + OFFSET.TEAM, 1),
        health = safeRead(readFloat, objAddress + OFFSET.HEALTH),
        maxHealth = safeRead(readFloat, objAddress + OFFSET.MAX_HEALTH),
        baseAtt = safeRead(readFloat, objAddress + OFFSET.BASE_ATT),
        bonusAtt = safeRead(readFloat, objAddress + OFFSET.BONUS_ATT),
        attRange = safeRead(readFloat, objAddress + OFFSET.ATT_RANGE),
        armor = safeRead(readFloat, objAddress + OFFSET.ARMOR),
        magRes = safeRead(readFloat, objAddress + OFFSET.MAG_RES),
        moveSpeed = safeRead(readFloat, objAddress + OFFSET.MOVE_SPEED),
        ap = safeRead(readFloat, objAddress + OFFSET.AP),
        level = safeRead(readFloat, objAddress + OFFSET.LEVEL),
        posX = safeRead(readFloat, objAddress + OFFSET.POS),
        posY = safeRead(readFloat, objAddress + OFFSET.POS + 4),
        posZ = safeRead(readFloat, objAddress + OFFSET.POS + 8),
        radius = safeRead(readFloat, objAddress + OFFSET.RADIUS),
        isAlive = safeRead(readBytes, objAddress + OFFSET.TARGETABLE, 1),
        targetable = safeRead(readBytes, objAddress + OFFSET.IS_ALIVE, 1),
        scale = safeRead(readFloat, objAddress + OFFSET.SCALE)
    }
end

-- 遍历对象列表的函数
local function getAllObjects(objListBase)
    local objects = {}
    if objListBase == nil then
        print("无法读取对象列表基址")
        return objects
    end

    for i = 0, 9999 do
        local currentObj = objListBase + i * 0x8
        local objAddress = readQword(currentObj)
        
        if objAddress ~= nil and objAddress ~= 0 then
            local objInfo = getObjectInfo(objAddress, i)
            if objInfo and objInfo.maxHealth == 3500 then
                table.insert(objects, objInfo)
            end
        end
    end
    
    return objects
end

-- 生成创建表的SQL语句
local function getCreateTableSQL()
    return [[
CREATE TABLE IF NOT EXISTS game_objects (
    object_index INT,
    name VARCHAR(64),
    team INT,
    health FLOAT,
    max_health FLOAT,
    base_attack FLOAT,
    bonus_attack FLOAT,
    ability_power FLOAT,
    attack_range FLOAT,
    armor FLOAT,
    magic_resist FLOAT,
    move_speed FLOAT,
    radius FLOAT,
    position_x FLOAT,
    position_y FLOAT,
    position_z FLOAT,
    scale FLOAT
);
]]
end

-- 生成插入数据的SQL语句
local function getInsertSQL(target)
    return string.format([[
INSERT INTO game_objects VALUES (
    %d,    -- object_index
    '%s',  -- name
    %d,    -- team
    %.2f,  -- health
    %.2f,  -- max_health
    %.2f,  -- base_attack
    %.2f,  -- bonus_attack
    %.2f,  -- ability_power
    %.2f,  -- attack_range
    %.2f,  -- armor
    %.2f,  -- magic_resist
    %.2f,  -- move_speed
    %.2f,  -- radius
    %.2f,  -- position_x
    %.2f,  -- position_y
    %.2f,  -- position_z
    %.2f   -- scale
);]], 
        target.index,
        target.name:gsub("'", "''"),
        target.team,
        target.health, target.maxHealth,
        target.baseAtt, target.bonusAtt,
        target.ap,
        target.attRange,
        target.armor,
        target.magRes,
        target.moveSpeed,
        target.radius,
        target.posX, target.posY, target.posZ,
        target.scale
    )
end

-- 主函数
local function main()
    local objects = getAllObjects(ADDR_OBJ_LIST)
    
    local output = getCreateTableSQL()
    output = output .. "\n-- 插入数据\n"
    
    for _, obj in ipairs(objects) do
        output = output .. getInsertSQL(obj)
    end
    
    print(output)
end

-- 执行主函数
main()

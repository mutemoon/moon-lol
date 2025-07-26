# base

GameTime = 0x1a9c6e0
LocalPlayer = 0x1ab8fd0
UnderMouseObject = 0x1a930c0
ObjectManager = 0x1a8fdd0
MinimapObject = 0x1a9c730
HeroList = 0x1a8fef0
MinionList = 0x1a92e98
MissileTree = 0x1a92eb0
TurretList = 0x1a9a498
ChatInterface = 0x1ab8ff0
ViewProjMatrices = 0x1b14b60
Renderer = 0x1b1e608
HudInstance = 0x1a8fde0

# object

ObjTeam = 0x249 （1字节）
ObjPos = 0x1f0 （Vec3 单浮点）
ObjVisible = 0x2f8 （1字节）
ObjMoveSpeed = 0x17dc （浮点）
ObjArmor = 0x17c4 （浮点）
ObjMagRes = 0x17cc （浮点）
ObjName = 0x4098 （字符串）
ObjBaseAtt = 0x179c （浮点）
ObjAttRange = 0x17e4 （浮点）
ObjAttSpeedMulti = 0x1798 （浮点）
ObjCrit = 0x17c0 （浮点）
ObjSummonerName = 0x42c8 （字符串）
SpellBook = 0x3b80 （指针）
BuffManager = 0x2e30 （指针）
ObjBonusAtt = 0x16f8 （浮点）
ObjAbilityPower = 0x1708 （浮点）
ObjMagPenRatio = 0x16a8 （浮点）
ObjLethality = 0x1778 （浮点）
ObjMagPen = 0x16a0 （浮点）
ObjArmPenRatio = 0x1604 （浮点）
ObjTargetable = 0xea0 （1字节）
ObjHealth = 0x1078 （浮点）
ObjMaxHealth = 0x10a0 （浮点）
ObjMana = 0x338 （浮点）
ObjMaxMana = 0x360 （浮点）
ObjLevel = 0x4cf0 （浮点）
ObjIndex = 0x10 （2字节）
ObjMagMulti = 0x170c （浮点）
ObjIsAlive = 0x248 （1字节）

# spell

SpellSlotLevel = 0x28 （浮点）
SpellSlotTime = 0x30 （浮点）
SpellInfo = 0x128 （指针）
SpellInfoName = 0x28

# buff

BuffManagerStart = 0x18
BuffManagerEnd = 0x20
BuffInfo = 0x10
BuffName = 0x8
BuffType = 0xc
BuffStartTime = 0x18
BuffEndTime = 0x1c
BuffCount = 0x38
BuffCountAlt1 = 0x8c
BuffCountAlt2 = 0x90

# missile

MissileName = 0x58
MissilePos = 0x24C
MissileSrcIdx = 0x348 # 0x34c
MissileDstIdx = 0x3b8
MissileStartPos = 0x378
MissileEndPos = 0x384

# gui

RendererWidth = 0xC
RendererHeight = 0x10
MinimapObjectHud = 0x288
MinimapHudPos = 0x68
MinimapHudSize = 0x70

# AiManager

AiManager = 0x41A8
AiManagerIsMoving = 0xA8
AiManagerDashSpeed = 0xA4
AiManagerEndPath = 0x34

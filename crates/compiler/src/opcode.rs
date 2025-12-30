/// 字节码指令 - 仿照 CPython
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
  // ============ 常量加载 ============
  /// 加载常量: LOAD_CONST index
  LoadConst = 0,
  
  // ============ 变量操作 ============
  /// 加载局部变量: LOAD_FAST index
  LoadFast = 10,
  /// 存储局部变量: STORE_FAST index
  StoreFast = 11,
  /// 加载全局变量: LOAD_GLOBAL index
  LoadGlobal = 12,
  /// 存储全局变量: STORE_GLOBAL index
  StoreGlobal = 13,
  /// 加载变量名: LOAD_NAME index
  LoadName = 14,
  /// 存储变量名: STORE_NAME index
  StoreName = 15,

  // ============ 栈操作 ============
  /// 弹出栈顶
  Pop = 20,
  /// 复制栈顶
  Dup = 21,
  /// 交换栈顶两个元素
  Swap = 22,

  // ============ 二元运算 ============
  BinaryAdd = 30,
  BinarySub = 31,
  BinaryMul = 32,
  BinaryDiv = 33,
  BinaryFloorDiv = 34,
  BinaryMod = 35,
  BinaryPow = 36,

  // ============ 一元运算 ============
  UnaryNeg = 40,
  UnaryNot = 41,
  UnaryPos = 42,

  // ============ 比较运算 ============
  CompareEq = 50,
  CompareNe = 51,
  CompareLt = 52,
  CompareLe = 53,
  CompareGt = 54,
  CompareGe = 55,

  // ============ 跳转指令 ============
  /// 无条件跳转: JUMP offset
  Jump = 60,
  /// 条件跳转(假): JUMP_IF_FALSE offset
  JumpIfFalse = 61,
  /// 条件跳转(真): JUMP_IF_TRUE offset
  JumpIfTrue = 62,
  /// 向后跳转(循环): LOOP offset
  Loop = 63,

  // ============ 函数相关 ============
  /// 调用函数: CALL argc
  Call = 70,
  /// 返回
  Return = 71,
  /// 创建函数: MAKE_FUNCTION
  MakeFunction = 72,

  // ============ 容器操作 ============
  /// 构建列表: BUILD_LIST count
  BuildList = 80,
  /// 构建字典: BUILD_DICT count
  BuildDict = 81,
  /// 构建元组: BUILD_TUPLE count
  BuildTuple = 82,
  /// 下标取值: BINARY_SUBSCR
  BinarySubscr = 83,
  /// 下标赋值: STORE_SUBSCR
  StoreSubscr = 84,

  // ============ 其他 ============
  /// 获取属性: GET_ATTR index
  GetAttr = 90,
  /// 设置属性: SET_ATTR index
  SetAttr = 91,
  /// 获取迭代器
  GetIter = 92,
  /// 迭代下一个
  ForIter = 93,

  /// 空操作
  Nop = 255,
}

impl From<u8> for OpCode {
  fn from(byte: u8) -> Self {
    unsafe { std::mem::transmute(byte) }
  }
}

impl From<OpCode> for u8 {
  fn from(op: OpCode) -> Self {
    op as u8
  }
}
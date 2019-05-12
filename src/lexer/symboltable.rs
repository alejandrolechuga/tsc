mod token;

use self::token:::Token;

struct SymbolTable {
    parent: Option<SymbolTable>,
    symbols: HashTable<&str, Token>,
}

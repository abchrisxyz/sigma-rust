
import Foundation
import ErgoLibC

/// Blockchain state (last headers, etc.)
class ErgoStateContext {
    internal var pointer: ErgoStateContextPtr
    
    /// Create new context
    init(preHeader : PreHeader, headers: BlockHeaders) throws {
        var ptr: ErgoStateContextPtr?
        let error = ergo_wallet_ergo_state_context_new(preHeader.pointer, headers.pointer, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    deinit {
        ergo_wallet_ergo_state_context_delete(self.pointer)
    }
}
extension ErgoStateContext: Equatable {
    static func ==(lhs: ErgoStateContext, rhs: ErgoStateContext) -> Bool {
        ergo_wallet_ergo_state_context_eq(lhs.pointer, rhs.pointer)
    }
}

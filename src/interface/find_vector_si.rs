use super::{CompileInput, CompileResult, Interface, InternResult, InternInput};

pub struct FindVectorSI {}

impl Interface for FindVectorSI {
    fn compile(&self, input: CompileInput) -> CompileResult {
        return Err(());
    }

    fn intern(&self, input: InternInput) -> InternResult {
        return Err(());
    }
}

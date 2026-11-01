//! Motor de LaTeX nativo portado de Tinta (MIT).
//!
//! - `ast`: árbol de nodos equivalente al `MNode` de Tinta.
//! - `macros`: expansión de `\newcommand`/`\def` locales a la ecuación.
//! - `parser`: LaTeX → AST.
//! - `layout`: AST → display list (`MathBox`).
//! - `render`: display list → egui.

pub mod ast;
pub mod layout;
pub mod macros;
pub mod parser;
pub mod render;

pub use ast::{Kind, MNode};
pub use layout::{layout, MathBox, MathLine, MathRule, MathRun, Measurer, Metrics};
pub use parser::parse_latex;
pub use render::EguiMeasurer;

use std::fs::File;
use std::io::{self, Write};

#[derive(Debug)]
struct TreeType {
    base_class_name: String,
    class_name: String,
    fields: Vec<String>,
}

pub fn generate_ast(output_dir: &str) -> io::Result<()> {
    define_ast(
        output_dir,
        "Expr",
        &["error", "token", "object"],
        &[
            "Assign   : Token name, Box<Expr> value",
            "Binary   : Box<Expr> left, Token operator, Box<Expr> right",
            "Grouping : Box<Expr> expression",
            "Literal  : Option<Object> value",
            "Unary    : Token operator, Box<Expr> right",
            "Variable : Token name",
        ],
    )?;
    define_ast(
        output_dir,
        "Stmt",
        &["error", "expr", "token"],
        &[
            "Expression : Expr expression",
            "Print      : Expr expression",
            "Var        : Token name, Option<Expr> initializer",
        ],
    )?;
    Ok(())
}

fn define_ast(
    output_dir: &str,
    base_name: &str,
    imports: &[&str],
    types: &[&str],
) -> io::Result<()> {
    let path = format!("{output_dir}/{}.rs", base_name.to_lowercase());
    let mut file = File::create(path)?;
    let mut tree_types = Vec::new();

    for i in imports {
        writeln!(file, "use crate::{}::*;", i)?;
    }

    for ttype in types {
        let (base_class_name, args) = ttype.split_once(":").unwrap();
        let class_name = format!("{}{}", base_class_name.trim(), base_name); // Binary + Expr
        let arg_split = args.split(',');
        let mut fields = Vec::new();
        for arg in arg_split {
            let (t2type, name) = arg.trim().split_once(" ").unwrap();
            fields.push(format!("{}: {}", name, t2type));
        }
        tree_types.push(TreeType {
            base_class_name: base_class_name.trim().to_string(),
            class_name,
            fields,
        });
    }

    writeln!(file, "\npub enum {base_name} {{")?;
    for t in &tree_types {
        writeln!(file, "    {}({}),", t.base_class_name, t.class_name)?;
    }
    writeln!(file, "}}\n")?;

    writeln!(file, "impl {} {{", base_name)?;
    writeln!(file, "    pub fn accept<T>(&self, {}_visitor: &dyn {base_name}Visitor<T>) -> Result<T, LoxError> {{", base_name.to_lowercase())?;
    writeln!(file, "        match self {{")?;
    for t in &tree_types {
        writeln!(
            file,
            "            {}::{}(v) => v.accept({}_visitor),",
            base_name,
            t.base_class_name,
            base_name.to_lowercase()
        )?;
    }
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}\n")?;

    for t in &tree_types {
        writeln!(file, "pub struct {} {{", t.class_name)?;
        for f in &t.fields {
            writeln!(file, "    pub {},", f)?;
        }
        writeln!(file, "}}\n")?;
    }

    writeln!(file, "pub trait {}Visitor<T> {{", base_name)?;
    for t in &tree_types {
        writeln!(
            file,
            "    fn visit_{}_{}(&self, expr: &{}) -> Result<T, LoxError>;",
            t.base_class_name.to_lowercase(),
            base_name.to_lowercase(),
            t.class_name
        )?;
    }
    writeln!(file, "}}\n")?;

    for t in &tree_types {
        writeln!(file, "impl {} {{", t.class_name)?;
        writeln!(
            file,
            "    pub fn accept<T>(&self, visitor: &dyn {}Visitor<T>) -> Result<T, LoxError> {{",
            base_name
        )?;
        writeln!(
            file,
            "        visitor.visit_{}_{}(self)",
            t.base_class_name.to_lowercase(),
            base_name.to_lowercase()
        )?;
        writeln!(file, "    }}")?;
        writeln!(file, "}}\n")?;
    }
    Ok(())
}

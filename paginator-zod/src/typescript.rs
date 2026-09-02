//! TypeScript Zod schema generation for the paginated response envelope.
//!
//! The output is plain Zod (targeting Zod v4 by default, matching `zod-rs-ts`)
//! and is therefore [Standard Schema](https://github.com/standard-schema/standard-schema)
//! compliant, so it works directly with TanStack Form, React Hook Form, and other
//! validation-library-agnostic tooling.

/// Which Zod import style to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZodVersion {
    /// `import * as z from "zod"` (Zod v4).
    #[default]
    V4,
    /// `import { z } from "zod"` (Zod v3).
    V3,
}

impl ZodVersion {
    fn import_line(self) -> &'static str {
        match self {
            ZodVersion::V4 => "import * as z from \"zod\";",
            ZodVersion::V3 => "import { z } from \"zod\";",
        }
    }
}

/// The `z.object({ ... })` expression for [`PaginatorResponseMeta`], without a
/// surrounding `export const`. Mirrors the Rust struct's serde output: `total`,
/// `total_pages`, `next_cursor`, and `prev_cursor` are optional because they are
/// skipped when `None`.
///
/// [`PaginatorResponseMeta`]: paginator_utils::PaginatorResponseMeta
pub fn meta_schema_expr() -> String {
    [
        "z.object({",
        "  page: z.number().int().nonnegative(),",
        "  per_page: z.number().int().positive(),",
        "  total: z.number().int().nonnegative().optional(),",
        "  total_pages: z.number().int().nonnegative().optional(),",
        "  has_next: z.boolean(),",
        "  has_prev: z.boolean(),",
        "  next_cursor: z.string().optional(),",
        "  prev_cursor: z.string().optional(),",
        "})",
    ]
    .join("\n")
}

/// A complete, importable TypeScript module exporting:
///
/// - `PaginationMetaSchema` and its inferred `PaginationMeta` type
/// - `paginated(item)`, a helper that wraps any item schema into the paginated
///   envelope, plus a `Paginated<T>` generic type
///
/// Use it like:
///
/// ```ts
/// import { paginated } from "./pagination";
/// const UsersPage = paginated(UserSchema);
/// type UsersPage = z.infer<typeof UsersPage>;
/// ```
pub fn response_module_ts() -> String {
    response_module_ts_with(ZodVersion::default())
}

/// Same as [`response_module_ts`] but with an explicit [`ZodVersion`].
pub fn response_module_ts_with(version: ZodVersion) -> String {
    let meta = indent(&meta_schema_expr(), 0);
    format!(
        r#"{import}

export const PaginationMetaSchema = {meta};

export type PaginationMeta = z.infer<typeof PaginationMetaSchema>;

export const paginated = <T extends z.ZodTypeAny>(item: T) =>
  z.object({{
    data: z.array(item),
    meta: PaginationMetaSchema,
  }});

export type Paginated<T> = {{
  data: T[];
  meta: PaginationMeta;
}};
"#,
        import = version.import_line(),
        meta = meta,
    )
}

fn indent(s: &str, spaces: usize) -> String {
    if spaces == 0 {
        return s.to_string();
    }
    let pad = " ".repeat(spaces);
    s.lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("{pad}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

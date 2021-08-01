use openapiv3::*;

pub fn ty_schema(ty: Type) -> Schema {
    Schema {
        schema_data: SchemaData {
            nullable: false,
            ..Default::default()
        },
        schema_kind: SchemaKind::Type(ty),
    }
}

//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.3

use super::sea_orm_active_enums::OrderStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "order")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub chain_id: i64,
    #[sea_orm(column_type = "VarBinary(StringLen::None)", unique)]
    pub order_id: Vec<u8>,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub user: Vec<u8>,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub filler: Vec<u8>,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub source_chain_selector: Vec<u8>,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub destination_chain_selector: Vec<u8>,
    pub sponsored: bool,
    pub primary_filler_deadline: DateTime,
    pub deadline: DateTime,
    #[sea_orm(column_type = "VarBinary(StringLen::None)", nullable)]
    pub call_recipient: Option<Vec<u8>>,
    #[sea_orm(column_type = "VarBinary(StringLen::None)", nullable)]
    pub call_data: Option<Vec<u8>>,
    pub order_status: OrderStatus,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

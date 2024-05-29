pub use sea_orm_migration::prelude::*;

mod m20240325_032714_create_event_table;
mod m20240325_032727_create_page_table;
mod m20240325_032732_create_post_table;
mod m20240325_032828_create_block_table;
mod m20240325_125942_create_index_at_post;
mod m20240407_034855_create_notion_database_table;
mod m20240512_100556_create_static_page_table;
mod m20240512_104914_rename_to_notion_parent_id_table;
mod m20240512_110241_add_parent_type_column;
mod m20240512_123038_remove_contents_column;
mod m20240517_085139_create_user_table;
mod m20240517_085140_create_prompt_session_table;
mod m20240517_085142_create_prompt_table;
mod m20240517_162442_create_prompt_page_table;
mod m20240526_114017_remove_fk_user_id;
mod m20240526_114430_remove_fk_prompt_session_id;
mod m20240526_114439_remove_fk_prompt_id;
mod m20240526_114446_remove_fk_page_id;
mod m20240528_111910_add_title_column;
mod m20240528_141124_add_tools_prompt;
mod m20240529_121200_add_draft_column;
mod m20240529_132201_create_nudge_table;
mod m20240529_134720_add_page_id_column;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240325_032714_create_event_table::Migration),
            Box::new(m20240325_032727_create_page_table::Migration),
            Box::new(m20240325_032732_create_post_table::Migration),
            Box::new(m20240325_032828_create_block_table::Migration),
            Box::new(m20240325_125942_create_index_at_post::Migration),
            Box::new(m20240407_034855_create_notion_database_table::Migration),
            Box::new(m20240512_100556_create_static_page_table::Migration),
            Box::new(
                m20240512_104914_rename_to_notion_parent_id_table::Migration,
            ),
            Box::new(m20240512_110241_add_parent_type_column::Migration),
            Box::new(m20240512_123038_remove_contents_column::Migration),
            Box::new(m20240517_085139_create_user_table::Migration),
            Box::new(m20240517_085140_create_prompt_session_table::Migration),
            Box::new(m20240517_085142_create_prompt_table::Migration),
            Box::new(m20240517_162442_create_prompt_page_table::Migration),
            Box::new(m20240526_114017_remove_fk_user_id::Migration),
            Box::new(m20240526_114430_remove_fk_prompt_session_id::Migration),
            Box::new(m20240526_114439_remove_fk_prompt_id::Migration),
            Box::new(m20240526_114446_remove_fk_page_id::Migration),
            Box::new(m20240528_111910_add_title_column::Migration),
            Box::new(m20240528_141124_add_tools_prompt::Migration),
            Box::new(m20240529_121200_add_draft_column::Migration),
            Box::new(m20240529_132201_create_nudge_table::Migration),
            Box::new(m20240529_134720_add_page_id_column::Migration),
        ]
    }
}

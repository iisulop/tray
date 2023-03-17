use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Poll {
    Table,
    Id,
    Title,
    CreationTime,
}

#[derive(Iden)]
enum Candidate {
    Table,
    Id,
    Url,
    PollId,
}

#[derive(Iden)]
enum Vote {
    Table,
    Id,
    CandidateId,
    CreationTime,
    SourceIp,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Poll::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Poll::CreationTime).date_time().not_null())
                    .col(
                        ColumnDef::new(Poll::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Poll::Title).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Candidate::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Candidate::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Candidate::PollId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-candidate-poll")
                            .from(Candidate::Table, Candidate::PollId)
                            .to(Poll::Table, Poll::Id),
                    )
                    .col(ColumnDef::new(Candidate::Url).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Vote::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Vote::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Vote::CandidateId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-vote-candidate")
                            .from(Vote::Table, Vote::CandidateId)
                            .to(Candidate::Table, Candidate::Id),
                    )
                    .col(ColumnDef::new(Vote::CreationTime).date_time().not_null())
                    .col(ColumnDef::new(Vote::SourceIp).string().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Poll::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Candidate::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Vote::Table).to_owned())
            .await?;
        Ok(())
    }
}

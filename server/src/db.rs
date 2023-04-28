use anyhow::anyhow;
use sqlx::{Pool, Sqlite};

use crate::schema::{CreateUpdateUser, User};

pub struct Database {
    pub db: Pool<Sqlite>,
}

impl Database {
    pub async fn create_user(&self, user: CreateUpdateUser) -> Result<User, anyhow::Error> {
        let mut tx = self.db.begin().await?;
        let new_id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Local::now().naive_local();

        sqlx::query!(
            r#"INSERT INTO User (id, name, birthday, address, activity, email, personal_phone,
                commercial_phone, uses_whatsapp, signed_at, updated_at, identities)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            new_id,
            user.name,
            user.birthday,
            user.address,
            user.activity,
            user.email,
            user.personal_phone,
            user.commercial_phone,
            user.uses_whatsapp,
            user.signed_at,
            created_at,
            user.identities,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(User {
            id: new_id,
            name: user.name,
            birthday: user.birthday,
            address: user.address,
            activity: user.activity,
            email: user.email,
            personal_phone: user.personal_phone,
            commercial_phone: user.commercial_phone,
            uses_whatsapp: user.uses_whatsapp,
            signed_at: user.signed_at,
            updated_at: created_at,
            identities: user.identities,
        })
    }

    pub async fn update_user(&self, user: CreateUpdateUser) -> Result<User, anyhow::Error> {
        let mut tx = self.db.begin().await?;
        let user_id = user
            .id
            .ok_or(anyhow!("User Id not present when updating."))?;

        let updated_at = chrono::Local::now().naive_local();

        sqlx::query!(
            r#"UPDATE User SET
                name=COALESCE(?, name),
                birthday=COALESCE(?, birthday),
                address=COALESCE(?, address),
                activity=COALESCE(?, activity),
                email=COALESCE(?, email),
                personal_phone=COALESCE(?, personal_phone),
                commercial_phone=COALESCE(?, commercial_phone),
                uses_whatsapp=COALESCE(?, uses_whatsapp),
                signed_at=COALESCE(?, signed_at),
                updated_at=COALESCE(?, updated_at),
                identities=COALESCE(?, identities)
            WHERE id = ?"#,
            user.name,
            user.birthday,
            user.address,
            user.activity,
            user.email,
            user.personal_phone,
            user.commercial_phone,
            user.uses_whatsapp,
            user.signed_at,
            updated_at,
            user.identities,
            user_id,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(User {
            id: user_id,
            name: user.name,
            birthday: user.birthday,
            address: user.address,
            activity: user.activity,
            email: user.email,
            personal_phone: user.personal_phone,
            commercial_phone: user.commercial_phone,
            uses_whatsapp: user.uses_whatsapp,
            signed_at: user.signed_at,
            updated_at: updated_at,
            identities: user.identities,
        })
    }

    pub async fn get_user(&self, id: String) -> Result<User, anyhow::Error> {
        let mut tx = self.db.begin().await?;

        let user = sqlx::query_as!(User, "SELECT * FROM User WHERE id=?", id)
            .fetch_one(&mut tx)
            .await?;
        Ok(user)
    }
}

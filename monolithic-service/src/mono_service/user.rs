use commons::*;
use mongo_svc::user::*;

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub password: String,
    pub favorite: i64,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone)]
pub struct UserService {
    pub name: String,
    user_db: Collection<Document>,
}

impl UserService {
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
            user_db: {
                let mut mongo_client_options = ClientOptions::parse(mongo_svc::URL).await?;
                mongo_client_options.app_name = Some(mono_svc::NAME.to_owned());
                let mongo_client = Client::with_options(mongo_client_options)?;
                mongo_client
                    .database(mongo_svc::DB)
                    .collection::<Document>(mongo_svc::coll::USER_PROFILE)
            },
        })
    }
    /// - Retrieve an optional user profile according to the given name.
    pub(in crate::mono_service) async fn retrieve_user_by_name(
        &self,
        name: &String,
    ) -> Option<UserProfile> {
        let filter = doc! {
            "username": name
        };
        let res = self.user_db.find_one(filter, None).await;
        match res {
            Ok(option) => match option {
                Some(doc) => match doc {
                    _ => match UserService::doc_to_user_profile(&doc) {
                        Ok(user_profile) => Some(user_profile),
                        Err(_) => None,
                    },
                },
                None => None,
            },
            Err(_) => None,
        }
    }
    /**
     * Retrieve all user profiles.
     * TODO: filter
     */
    async fn _retrieve_user_all(&self) -> Option<Vec<Option<UserProfile>>> {
        let filter = doc! {};
        let mut cursor = match self.user_db.find(filter, None).await {
            Ok(cursor) => cursor,
            Err(_) => return None,
        };
        let mut user_list = Vec::<Option<UserProfile>>::new();
        loop {
            match cursor.try_next().await {
                Ok(opt) => match opt {
                    Some(doc) => match UserService::doc_to_user_profile(&doc) {
                        Ok(user_profile) => user_list.push(Some(user_profile)),
                        Err(_) => user_list.push(None),
                    },
                    None => break,
                },
                Err(_) => break,
            }
        }
        Some(user_list)
    }

    fn _user_profile_to_doc(user_profile: &UserProfile) -> Document {
        doc! {
            USERNAME: &user_profile.username,
            PASSWORD: &user_profile.password,
            FAVORITE: &user_profile.favorite,
            LATITUDE: &user_profile.latitude,
            LONGITUDE: &user_profile.longitude,
        }
    }

    pub(in crate::mono_service) fn doc_to_user_profile(
        document: &Document,
    ) -> Result<UserProfile, ValueAccessError> {
        Ok(UserProfile {
            id: document.get_object_id(DOC_ID)?.to_hex(),
            username: document.get_str(USERNAME)?.to_owned(),
            password: document.get_str(PASSWORD)?.to_owned(),
            favorite: document.get_i64(FAVORITE)?,
            latitude: document.get_f64(LATITUDE)?,
            longitude: document.get_f64(LONGITUDE)?,
        })
    }
}

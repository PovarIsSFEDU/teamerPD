use mongodb::{Client, Collection};
use mongodb::bson::doc;
use crate::database::{RegistrationResult, LoginError, User, VerificationError, DatabaseError, UserDataType, TeamDataType, TeamCreationError, GetTeamError};
use crate::auth::{RegistrationData, LoginData};
use crate::prelude::MapBoth;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::marker::Send;
use rocket::futures::StreamExt;
use crate::database::new_user::NewUser;
use crate::database::team::Team;
use crate::teams::TeamType;
pub type DatabaseOperationResult = Result<(), DatabaseError>;

macro_rules! generate_getter {
    (name: $name: ident, database: $database: literal, collection: $collection: literal) => {
        pub async fn $name<T>(&self, field: &str, value: &str) -> mongodb::error::Result<Option<T>>
            where T: DeserializeOwned + Unpin + Send + Sync
        {
            let db = self.client.database($database).collection::<T>($collection);
            db.find_one(doc! {field: value}, None).await
        }
    };
}


pub struct MongoDriver {
    client: Client,
}

impl MongoDriver {
    pub fn new(client: Client) -> Self {
        MongoDriver {
            client
        }
    }

    pub async fn validate_registration(&self, data: &RegistrationData) -> RegistrationResult {
        let found = self.get::<RegistrationData>("login", data.login()).await;

        match found {
            Ok(None) => {
                let email = self.get::<RegistrationData>("email", data.email()).await;
                match email {
                    Ok(None) => RegistrationResult::Ok,
                    Ok(Some(_)) => RegistrationResult::Exists,
                    Err(_) => RegistrationResult::Other
                }
            }

            Ok(Some(_)) => RegistrationResult::Exists,
            Err(_) => RegistrationResult::Other
        }
    }

    pub async fn register(&self, data: RegistrationData) -> Result<User, RegistrationResult> {
        let db = self.client.database("user").collection::<NewUser>("login");
        let found = self.get::<LoginData>("login", data.login()).await;
        match found {
            Err(_) => return Err(RegistrationResult::Other),
            Ok(Some(_)) => return Err(RegistrationResult::Exists),
            Ok(None) => {}
        }

        let found = self.get::<LoginData>("email", data.email()).await;
        match found {
            Err(_) => return Err(RegistrationResult::Other),
            Ok(Some(_)) => return Err(RegistrationResult::Exists),
            Ok(None) => {}
        }

        let insert = NewUser::from(data.clone());
        let _ = db
            .insert_one(insert, None)
            .await;
        let db = self.client.database("user").collection::<User>("users");
        let result = db
            .insert_one(User::from(&data), None)
            .await;

        match result {
            Ok(_) => Ok(User::from(&data)),
            Err(_) => Err(RegistrationResult::Other)
        }
    }

    pub async fn validate_login(&self, data: LoginData) -> Result<User, LoginError> {
        let found = self.get::<LoginData>("login", data.login()).await;

        match found {
            Ok(Some(result)) => {
                let matches = bcrypt::verify(data.password(), result.password()).unwrap_or(false);

                match matches {
                    true => {
                        let user = self.client
                            .database("user")
                            .collection::<User>("users")
                            .find_one(doc! {"login": data.login()}, None)
                            .await
                            .unwrap()
                            .unwrap();

                        Ok(user)
                    }

                    false => Err(LoginError::IncorrectPassword)
                }
            }

            Ok(None) => Err(LoginError::NotExist),
            Err(_) => Err(LoginError::Other)
        }
    }

    pub async fn verify_email(&self, key: String, login: String) -> Result<(), VerificationError> {
        let collection = self.client.database("user").collection::<LoginData>("login");

        let filter = doc! {"login": login, "verification_key": key, "is_verified": false};
        let modification = doc! {"$set": {"is_verified": true}, "$unset": {"verification_key": ""}};

        let result = collection.update_one(filter, modification, None).await;

        match result {
            Ok(result) if result.matched_count > 0 => Ok(()),
            Ok(_) => Err(VerificationError::AlreadyVerified),
            Err(_) => Err(VerificationError::Other)
        }
    }

    pub async fn get_verification_key(&self, login: String) -> Result<(String, String), DatabaseError> {
        #[derive(Deserialize)]
        struct VKey {
            #[serde(alias = "verification_key")] value: String,
            email: String,
        }

        let key = self.get::<VKey>("login", &login).await;
        match key {
            Ok(Some(key)) => Ok((key.email, key.value)),
            Ok(None) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn set_recovery_key(&self, user: &RegistrationData, key: &str) -> DatabaseOperationResult {
        let collection = self.get_login_collection::<LoginData>();
        let filter = doc! {"login": user.login()};
        let modification = doc! {"$set": {"recovery_key": key}};
        let update_result = collection.update_one(filter, modification, None).await;

        update_result.map_both(|_| (), |_| DatabaseError::Other)
    }

    pub async fn validate_recovery(&self, key: String, user: LoginData) -> DatabaseOperationResult {
        let collection = self.get_login_collection::<LoginData>();
        let filter = doc! {"login": user.login(), "recovery_key": key};
        let found = collection.find_one(filter.clone(), None).await;
        match found {
            Ok(Some(_)) => {
                let new_pass = bcrypt::hash(user.password(), 5).unwrap();
                let modification = doc! {"$unset": {"recovery_key": ""}, "$set": {"password": new_pass}};
                let result = collection.update_one(filter, modification, None).await;

                result.map_both(|_| (), |_| DatabaseError::Other)
            }

            Ok(None) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn set_user_data(&self, data_type: UserDataType, login: &str, value: &str) -> DatabaseOperationResult {
        let collection = self.client.database("user").collection::<User>("users");
        let parameter = match data_type {
            UserDataType::Photo => "photo",
            UserDataType::Resume => "resume",
            UserDataType::TeamName => "team",
            UserDataType::Email => "email",
            UserDataType::AdminStatus => "adm",
            UserDataType::Competences => "competences"
        };

        let filter = doc! {"login": login};
        let update = doc! {"$set": {parameter: value}};
        let result = collection.update_one(filter, update, None).await;

        match result {
            Ok(result) if result.modified_count > 0 => Ok(()),
            Ok(_) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn update_user(&self, user: User) -> DatabaseOperationResult {
        let collection = self.client.database("user").collection::<User>("users");
        let filter = doc! {"login": user.login.clone()};
        let update = doc! {"$set": {
            "login": user.login,
            "name": user.name,
            "surname": user.surname,
            "team": user.team,
            "photo": user.photo,
            "resume": user.resume,
            "adm": user.adm,
            "email": user.email,
            "competences": user.competences
        }};

        let result = collection.update_one(filter, update, None).await;

        match result {
            Ok(result) if result.modified_count > 0 => Ok(()),
            Ok(_) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn update_competences(&self, login: &str, value: &Vec<String>) -> DatabaseOperationResult {
        let collection = self.client.database("user").collection::<User>("users");
        let filter = doc! {"login": login};
        let update = doc! {"$set": {"competences": value}};

        let result = collection.update_one(filter, update, None).await;

        match result {
            Ok(result) if result.modified_count > 0 => Ok(()),
            Ok(_) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn set_team_data(&self, data_type: TeamDataType, name: &str, file_name: &str) -> DatabaseOperationResult {
        let collection = self.client.database("teams").collection::<Team>("teams");
        let parameter = match data_type {
            TeamDataType::Name => "name",
            TeamDataType::Logo => "logo"
        };

        let filter = doc! {"name": name};
        let update = doc! {"$set": {parameter: file_name}};
        let result = collection.update_one(filter, update, None).await;

        match result {
            Ok(result) if result.modified_count > 0 => Ok(()),
            Ok(_) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn get_user_team(&self, _team_type: TeamType, username: &String) -> Result<String, GetTeamError> {
        let collection = self.client.database("user").collection::<User>("users");
        let filter = doc! {"login": username};
        let result = collection.find_one(filter, None).await;

        match result {
            Ok(Some(user)) => match user.team {
                Some(team) => Ok(team),
                None => Err(GetTeamError::NotInTeam)
            }

            Ok(None) => Err(GetTeamError::NotFound),
            Err(_) => Err(GetTeamError::Other)
        }
    }

    pub async fn create_team(&self, _team_type: TeamType, team_name: String, captain: String) -> Result<Team, TeamCreationError> {
        let db = self.client.database("teams").collection::<Team>("teams");
        let team = Team::new(team_name, captain);
        //TODO: Сделать проверку наличия комманды с таким же названием
        let result = db
            .insert_one(team.clone(), None)
            .await;

        match result {
            Ok(_) => Ok(team),
            Err(_) => Err(TeamCreationError::Other)
        }
    }

    pub async fn add_team_member(&self, team: &str, user: &str) -> DatabaseOperationResult {
        let members = self.get_team::<Team>("name", team).await;
        let mut members = match members {
            Ok(Some(t)) => t,
            Ok(None) => return Err(DatabaseError::NotFound),
            Err(_) => return Err(DatabaseError::Other)
        }.members;

        members.push(user.to_owned());
        let update_result = self
            .client
            .database("teams")
            .collection::<Team>("teams")
            .update_one(doc!{"name": team.clone()}, doc!{"$set": {"members": members}}, None)
            .await;

        match update_result {
            Ok(result) if result.modified_count > 0 => {
                self.set_user_data(UserDataType::TeamName, user, team);
                Ok(())
            }
            Ok(_) => Err(DatabaseError::NotFound),
            Err(e) => Err(DatabaseError::Other)
        }
    }

    pub async fn get_teams(&self) -> Result<Vec<Team>, DatabaseError> {
        let db = self.client.database("teams").collection::<Team>("teams");
        let teams = db.find(doc!{}, None).await;
        match teams {
            Err(_) => Err(DatabaseError::Other),
            Ok(cursor) => Ok(
                cursor
                    .take(10)
                    .map(|x| x.unwrap_or(Team::default()))
                    .collect::<Vec<_>>()
                    .await
            )
        }
    }

    pub async fn get<T>(&self, field: &str, value: &str) -> mongodb::error::Result<Option<T>>
        where T: DeserializeOwned + Unpin + Send + Sync
    {
        let db = self.get_login_collection::<T>();
        db.find_one(doc! {field: value.to_string()}, None).await
    }

    pub async fn get_by_login<T>(&self, value: &str) -> mongodb::error::Result<Option<T>>
        where T: DeserializeOwned + Unpin + Send + Sync
    {
        let db = self.client.database("user").collection::<T>("users");
        db.find_one(doc! {"login": value}, None).await
    }

    pub async fn get_user<T>(&self, field: &str, value: &str) -> mongodb::error::Result<Option<T>>
        where T: DeserializeOwned + Unpin + Send + Sync
    {
        let db = self.client.database("user").collection::<T>("users");
        db.find_one(doc! {field: value}, None).await
    }

    generate_getter! {
        name: get_team,
        database: "teams",
        collection: "teams"
    }

    fn get_login_collection<T>(&self) -> Collection<T>
        where T: DeserializeOwned + Unpin + Send + Sync
    {
        self.client.database("user").collection::<T>("login")
    }
}
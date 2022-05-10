use mongodb::{Client, Collection};
use mongodb::bson::{Bson, bson, doc, Regex};
use crate::database::{RegistrationResult, LoginError, User, VerificationError, DatabaseError, UserDataType, TeamDataType, TeamCreationError, GetTeamError};
use crate::auth::{RegistrationData, LoginData};
use crate::prelude::MapBoth;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::marker::Send;
use mongodb::bson::Bson::Document;
use mongodb::options::FindOptions;
use rocket::futures::{future, Stream, StreamExt};
use syn::__private::str;
use crate::database::new_user::NewUser;
use crate::database::team::Team;
use crate::teams::TeamType;
use crate::database::AddUserToTeamResult;
use crate::database::notification::Notification;
use crate::database::task::Task;

use crate::prelude::concat::Concatenate;

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

        let result = match (data_type, value) {
            (UserDataType::TeamName, "") => {
                let update = doc! {"$set": {parameter: None::<String>}};
                collection.update_one(filter, update, None).await
            }
            _ => {
                let update = doc! {"$set": {parameter: value}};
                collection.update_one(filter, update, None).await
            }
        };


        match result {
            Ok(result) if result.modified_count > 0 => Ok(()),
            Ok(_) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn update_user(&self, user: User) -> DatabaseOperationResult {
        let collection = self.client.database("user").collection::<User>("users");
        let filter = doc! {"login": user.login.clone()};
        let update = doc! {"$set":{
            "login": user.login,
            "name": user.name,
            "surname": user.surname,
            "city": user.city,
            "bio": user.bio,
            "tg": user.tg,
            "git": user.git,
            "level": user.level,
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

    #[allow(dead_code)]
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
    async fn update_user_team(&self, team_name: &String, username: &String) -> DatabaseOperationResult
    {
        let user_coll = self.client.database("user").collection::<User>("users");
        let user_filter = doc! {"login": username};
        let user_update = doc! {"$set": {"team": team_name}};
        let user_result = user_coll.update_one(user_filter, user_update, None).await;
        match user_result {
            Ok(result) if result.modified_count > 0 => Ok(()),
            Ok(_) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }
    async fn add_user_to_vector_team(&self, team_name: &String, username: &String) -> DatabaseOperationResult
    {
        let team_coll = self.client.database("teams").collection::<Team>("teams");
        let team_filter = doc! {"name": team_name};
        let team_result = team_coll.find_one(team_filter.clone(), None).await;
        match team_result {
            Ok(Some(team)) => {
                let mut members = team.members.clone();
                members.push(username.clone());
                let team_update = doc! {"$set":{"members": members}};
                let team_result_2 = team_coll.update_one(team_filter, team_update, None).await;
                match team_result_2 {
                    Ok(result) if result.modified_count > 0 => Ok(()),
                    Ok(_) => Err(DatabaseError::NotFound),
                    Err(_) => Err(DatabaseError::Other)
                }
            },
            Ok(None) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn add_user_to_team(&self, team_name: &String, username: &String) -> AddUserToTeamResult {
        let user_coll = self.client.database("user").collection::<User>("users");
        let user_filter = doc! {"login": username};
        let user_result = user_coll.count_documents(user_filter, None).await;
        let team_coll = self.client.database("teams").collection::<Team>("teams");
        let team_filter = doc! {"name": team_name};
        let team_result = team_coll.count_documents(team_filter, None).await;
        match user_result {
            Ok(res) if res > 0 => {},
            Ok(_) =>  {
                return AddUserToTeamResult::UserNotFound},
            Err(_) => return AddUserToTeamResult::Error
        }
        match team_result {
            Ok(res) if res > 0 => {},
            Ok(_) => {
                return AddUserToTeamResult::TeamNotFound
            },
            Err(_) => return AddUserToTeamResult::Error
        }
        let upd_res_user = self.update_user_team(team_name, username).await;
        match upd_res_user
        {
            Ok(_) => {},
            Err(_) => return AddUserToTeamResult::Error
        }
        let upd_res_team = self.add_user_to_vector_team(team_name, username).await;
        match upd_res_team
        {
            Ok(_) => {},
            Err(_) => return AddUserToTeamResult::Error
        }
        AddUserToTeamResult::Ok
    }

    pub async fn check_is_captain(&self, team_name: &String, captain: &String) -> Result<bool, DatabaseError>
    {
        let team_coll = self.client.database("teams").collection::<Team>("teams");
        let team_filter = doc! {"name": team_name, "captain": captain};
        let team_result = team_coll.count_documents(team_filter.clone(), None).await;
        match team_result {
            Ok(res) if res > 0 => Ok(true),
            Ok(_) => Ok(false),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn create_team(&self, _team_type: TeamType, mut team: Team, captain: String) -> Result<Team, TeamCreationError> {
        let db = self.client.database("teams").collection::<Team>("teams");
        team.captain = captain.clone();
        team.members.push(captain);
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

        if members.contains(&user.to_owned()) {
            return Err(DatabaseError::Other);
        }

        members.push(user.to_owned());
        let update_result = self
            .client
            .database("teams")
            .collection::<Team>("teams")
            .update_one(doc!{"name": team.clone()}, doc!{"$set": {"members": members}}, None)
            .await;

        match update_result {
            Ok(result) if result.modified_count > 0 => {
                let _ = self.set_user_data(UserDataType::TeamName, user, team).await;
                Ok(())
            }
            Ok(_) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn remove_team_member(&self, team: &str, user: &str) -> DatabaseOperationResult {
        let db_team = self.get_team::<Team>("name", team).await;
        let db_team = match db_team {
            Ok(Some(t)) => t,
            Ok(None) => return Err(DatabaseError::NotFound),
            Err(_) => return Err(DatabaseError::Other)
        };
        let members = db_team.clone().members;
        if user.to_owned() == db_team.captain {
            for member in members {
                self.set_user_data(UserDataType::TeamName, &member, "").await?;
            }

            self
                .client
                .database("teams")
                .collection::<Team>("teams")
                .delete_one(doc! {"name": team}, None)
                .await?;
            return Ok(());
        }

        if !members.contains(&user.to_owned()) {
            return Err(DatabaseError::Other);
        }

        let members = members.into_iter().filter(|x| !x.eq(user) ).collect::<Vec<String>>();
        let update_result = self
            .client
            .database("teams")
            .collection::<Team>("teams")
            .update_one(doc!{"name": team.clone()}, doc!{"$set": {"members": members}}, None)
            .await;

        match update_result {
            Ok(result) if result.modified_count > 0 => {
                let _ = self.set_user_data(UserDataType::TeamName, user, "").await;
                Ok(())
            }
            Ok(_) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn create_task(&self, task: Task, team: &str) -> DatabaseOperationResult {
        let tasks = self.client.database("teams").collection::<Task>(team);
        let result = tasks.insert_one(task, None).await;

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn update_task(&self, team: &str, task: Task) -> DatabaseOperationResult {
        let tasks = self.client.database("teams").collection::<Task>(team);
        let update = doc! { "$set": {
            "name": &task.name,
            "description": &task.description,
            "status": format!("{:?}", task.status),
            "assignee": &task.assignee,
            "priority": format!("{:?}", task.priority),
            "expiration": &task.expiration
        }};
        let update_result = tasks.update_one(doc! {"name": &task.name}, update, None).await;

        println!("{:?}", update_result);
        match update_result {
            Ok(result) if result.modified_count > 0 => Ok(()),
            Ok(e) => { println!("Modified {}", e.modified_count); Err(DatabaseError::NotFound) },
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn remove_task(&self, team: &str, task: Task) -> DatabaseOperationResult {
        let tasks = self.client.database("teams").collection::<Task>(team);

        let delete_result = tasks.delete_one(doc! {"name": task.name}, None).await;
        match delete_result {
            Ok(result) if result.deleted_count > 0 => Ok(()),
            Ok(_) => Err(DatabaseError::NotFound),
            Err(_) => Err(DatabaseError::Other)
        }
    }

    pub async fn get_teams(&self, offset: usize, limit: usize) -> Result<Vec<Team>, DatabaseError> {
        let db = self.client.database("teams").collection::<Team>("teams");
        let teams = db.find(doc! {}, None).await;
        match teams {
            Err(_) => Err(DatabaseError::Other),
            Ok(cursor) => Ok(
                cursor
                    .skip((offset - 1) * 6)
                    .take(limit)
                    .map(|x| x.unwrap_or(Team::default()))
                    .collect::<Vec<_>>()
                    .await
            )
        }
    }

    pub async fn get_teams_pages(&self) -> Result<u64, DatabaseError> {
        let db = self.client.database("teams").collection::<User>("teams");
        let teams_count = db.count_documents(doc! {}, None).await;
        match teams_count {
            Err(_) => Err(DatabaseError::Other),
            Ok(res) => {
                if res < 6 { Ok(1) } else if res % 6 == 0 { Ok(res / 6) } else { Ok(res / 6 + 1) }
            }
        }
    }

    pub async fn get_users(&self, offset: usize, limit: usize) -> Result<Vec<User>, DatabaseError> {
        let db = self.client.database("user").collection::<User>("users");
        let users = db.find(doc! {"surname": { "$ne": "" }}, None).await;

        match users {
            Err(_) => Err(DatabaseError::Other),
            Ok(cursor) => Ok(
                cursor
                    .skip((offset - 1) * 5)
                    .take(limit)
                    .map(|x| x.unwrap_or(User::default()))
                    .collect::<Vec<_>>()
                    .await
            )
        }
    }

    pub async fn get_users_pages(&self) -> Result<u64, DatabaseError> {
        let db = self.client.database("user").collection::<User>("users");
        let pages_count = db.count_documents(doc! {"surname": { "$ne": "" }}, None).await;
        match pages_count {
            Err(_) => Err(DatabaseError::Other),
            Ok(res) => {
                if res < 5 { Ok(1) } else if res % 5 == 0 { Ok(res / 5) } else { Ok(res / 5 + 1) }
            }
        }
    }

    pub async fn find_by_username(&self, param: String) -> Result<Vec<User>, DatabaseError> {
        let db = self.client.database("user").collection::<User>("users");
        let reg = Regex{ pattern: param, options: "i".to_string() };
        let users = db.find(doc! {"$or": [  { "login": &reg } , { "name": &reg }, { "surname": &reg } ]}, None).await;
        match users {
            Err(_) => Err(DatabaseError::Other),
            Ok(cursor) => Ok({
                println!("{:?}", cursor.size_hint());
                cursor
                    .take(5)
                    .map(|x| x.unwrap_or(User::default()))
                    .collect::<Vec<_>>()
                    .await
            }
            )
        }
    }

    pub async fn check_notifications(&self, user: String) -> Result<Vec<Notification>, DatabaseError> {
        let db = self.client.database("notifications").collection::<Notification>(&user);
        let notifications = db.find(doc! {"seen": false}, None).await;
        let vec = match notifications {
            Err(_) => return Err(DatabaseError::Other),
            Ok(cursor) => {
                let result = cursor
                    .filter(|x| future::ready(x.is_ok()))
                    .map(|x| x.unwrap())
                    .collect::<Vec<Notification>>()
                    .await;

                db.update_many(
                    doc! {"seen": false},
                    doc! {"$set": { "seen": true }},
                    None
                ).await?;
                result
            }
        };

        #[derive(Deserialize)]
        struct Seen {
            #[serde(default)]
            pub seen: Vec<i32>
        }
        let db = self.client.database("notifications").collection::<Notification>("common");
        let user_db = self.client.database("user").collection::<Seen>("users");
        let seen = user_db
            .find_one(doc! {"login": &user}, None)
            .await?
            .unwrap()
            .seen;

        let common = db
            .find(doc! {"_id": {"$nin": &seen }}, None)
            .await?
            .map(|x| x.unwrap())
            .collect::<Vec<Notification>>()
            .await;

        for Notification {id, ..} in &common {
            user_db.update_one(doc! {"login": &user}, doc! {"$push": {"seen": id}}, None).await?;
        }

        Ok(vec.into_iter().chain(common).collect())
    }

    pub async fn send_notification(&self, user: &str, header: String, body: String) -> DatabaseOperationResult {
        let db = self.client.database("notifications").collection::<Notification>(&user);
        db.insert_one(Notification::new(header, body), None).await?;

        Ok(())
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
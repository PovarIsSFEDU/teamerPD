use mongodb::bson::doc;
use rocket::http::Status;
use rocket::State;
use crate::auth::token::Token;
use crate::database::GetTeamError;
use crate::MongoDriver;
use crate::database::team::Team;
use crate::teams::TeamType;

#[get("/my_team")]
pub async fn my_team(token: Token, db: &State<MongoDriver>) -> Result<String, Status> {
    let user = token.claims.iss;
    let team = db.get_user_team(TeamType::Hackathon, &user).await;

    match team {
        Err(GetTeamError::NotFound) => Err(Status::NotFound),
        Err(GetTeamError::NotInTeam) => Err(Status::NoContent),
        Err(GetTeamError::Other) => Err(Status::InternalServerError),
        Ok(t) => {
            let team = db
                .get_team::<Team>("name", &t)
                .await
                .unwrap()
                .unwrap();

            Ok(serde_json::to_string(&team).unwrap())
        }
    }
}
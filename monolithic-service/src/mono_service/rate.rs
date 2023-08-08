use super::geo::HotelInfo;
use commons::*;
use mongo_svc::hotel::*;

#[derive(Debug, Clone)]
pub struct RateService {
    pub name: String,
    hotel_db: Collection<Document>,
}

impl RateService {
    /// - Create a new rate service connected to the database.
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
            hotel_db: {
                let mut mongo_client_options = ClientOptions::parse(mongo_svc::URL).await?;
                mongo_client_options.app_name = Some(rate_svc::NAME.to_owned());
                let mongo_client = Client::with_options(mongo_client_options)?;
                mongo_client
                    .database(mongo_svc::DB)
                    .collection::<Document>(mongo_svc::coll::HOTEL)
            },
        })
    }

    pub(in crate::mono_service) async fn retrieve_hotel_all(&self) -> Option<Vec<HotelInfo>> {
        let filter = doc! {};
        let mut cursor = match self.hotel_db.find(filter, None).await {
            Ok(cursor) => cursor,
            Err(_) => return None,
        };
        let mut hotel_list = Vec::<HotelInfo>::new();
        loop {
            match cursor.try_next().await {
                Ok(opt) => match opt {
                    Some(doc) => match RateService::doc_to_hotel_info(&doc) {
                        Ok(hotel_info) => hotel_list.push(hotel_info),
                        Err(_) => (),
                    },
                    None => break,
                },
                Err(_) => break,
            }
        }
        Some(hotel_list)
    }

    fn doc_to_hotel_info(document: &Document) -> Result<HotelInfo, ValueAccessError> {
        Ok(HotelInfo {
            id: document.get_object_id(DOC_ID)?.to_hex(),
            name: document.get_str(NAME)?.to_owned(),
            latitude: document.get_f64(LATITUDE)?,
            longitude: document.get_f64(LONGITUDE)?,
            provide: document.get_i64(PROVIDE)?,
        })
    }
    /// - Check if a hotel satisfies the user's requirement.
    pub(in crate::mono_service) fn check_satisfy(favorite: i64, provide: i64) -> bool {
        let (res, _) = favorite.overflowing_shr((provide - 1) as u32);
        if favorite == 0 || res & 1 == 1 {
            true
        } else {
            false
        }
    }
}

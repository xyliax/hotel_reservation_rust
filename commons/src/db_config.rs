pub mod mongo_svc {
    pub const URL: &'static str = "mongodb://localhost:27017";
    pub const DB: &'static str = "hotel_app";
    pub mod coll {
        pub const USER_PROFILE: &'static str = "user_profile";
        pub const COMMENT: &'static str = "comment";
        pub const HOTEL: &'static str = "hotel";
        pub const RATE_PLAN: &'static str = "rate_plan";
    }
    pub mod user {
        pub const DOC_ID: &'static str = "_id";
        pub const STR_ID: &'static str = "id";
        pub const USERNAME: &'static str = "username";
        pub const PASSWORD: &'static str = "password";
        pub const FAVORITE: &'static str = "favorite";
        pub const LATITUDE: &'static str = "latitude";
        pub const LONGITUDE: &'static str = "longitude";
    }
    pub mod hotel {
        pub const DOC_ID: &'static str = "_id";
        pub const STR_ID: &'static str = "id";
        pub const NAME: &'static str = "name";
        pub const LATITUDE: &'static str = "latitude";
        pub const LONGITUDE: &'static str = "longitude";
        pub const PROVIDE: &'static str = "provide";
    }
    pub mod comment {
        pub const DOC_ID: &'static str = "_id";
        pub const STR_ID: &'static str = "id";
        pub const HOTEL_ID: &'static str = "hotel_id";
        pub const TEXT: &'static str = "text";
        pub const DATE: &'static str = "date";
        pub const AUTHOR_ID: &'static str = "author_id";
    }
    pub mod rate_plan {
        pub const DOC_ID: &'static str = "_id";
        pub const STR_ID: &'static str = "id";
        pub const IN_DATE: &'static str = "in_date";
        pub const DURATION: &'static str = "duration";
        pub const DESCRIPTION: &'static str = "description";
    }
}

pub mod redis_svc {
    pub const USER: i8 = 0;
    pub const PROFILE: i8 = 1;
}

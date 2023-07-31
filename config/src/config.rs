pub mod user_svc {
    pub const NAME: &'static str = "srv-user";
    pub const ADDR: &'static str = "[::1]:50051";
    pub const PROT: &'static str = "http://[::1]:50051";
}

pub mod reserv_svc {
    pub const NAME: &'static str = "srv-reservation";
    pub const ADDR: &'static str = "[::1]:50052";
    pub const PROT: &'static str = "http://[::1]:50052";
}

pub mod profile_svc {
    pub const NAME: &'static str = "srv-profile";
    pub const ADDR: &'static str = "[::1]:50053";
    pub const PROT: &'static str = "http://[::1]:50053";
}

pub mod recomm_svc {
    pub const NAME: &'static str = "srv-recommendation";
    pub const ADDR: &'static str = "[::1]:50054";
    pub const PROT: &'static str = "http://[::1]:50054";
}

pub mod mongo_svc {
    pub const URL: &'static str = "mongodb://localhost:27017";
    pub const DB: &'static str = "hotel_app";
    pub mod coll {
        pub const USER_PROFILE: &'static str = "user_profile";
        pub const COMMENT: &'static str = "comment";
    }
    pub mod user {
        pub const DOC_ID: &'static str = "_id";
        pub const STR_ID: &'static str = "id";
        pub const USERNAME: &'static str = "username";
        pub const PASSWORD: &'static str = "password";
        pub const LOCATION: &'static str = "location";
        pub const FAVORITE: &'static str = "favorite";
    }
    pub mod comment {
        pub const DOC_ID: &'static str = "_id";
        pub const STR_ID: &'static str = "id";
        pub const HOTEL_ID: &'static str = "hotel_id";
        pub const TEXT: &'static str = "text";
        pub const DATE: &'static str = "date";
        pub const AUTHOR: &'static str = "author";
    }
}

pub mod redis_svc {
    pub const USER: i8 = 0;
    pub const PROFILE: i8 = 1;
}

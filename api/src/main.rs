fn main() {
    // User Table
    // (unique) | (unique) | (unique) | Date | Date
    // id | username | email | created_at | updated_at

    // Repository Table | Many to one with User
    // (unique) | (unique) | string | (unique, AWS S3 public-read bucket url) | Date | Date
    // id | name | description | url | created_at | updated_at

    // Basic user management (Sync between database and AWS Cognito)
    // - Register
    // - Login
    // - Logout
    // - Update
    // - Delete (also delete all repos)

    // Basic repository management
    // - Create
    // - Update
    // - Delete
}

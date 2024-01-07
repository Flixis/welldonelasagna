CREATE TABLE wdl_database.discord_messages (
    Id BIGINT AUTO_INCREMENT PRIMARY KEY,
    MessageId BIGINT,
    ChannelId BIGINT,
    UserId BIGINT,
    Content TEXT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
    Timestamp TIMESTAMP,
    PremiumType VARCHAR(50)
);

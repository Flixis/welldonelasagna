USE wdl_database;

DROP TABLE IF EXISTS discord_messages;

CREATE TABLE discord_messages (
    Id BIGINT AUTO_INCREMENT PRIMARY KEY,
    MessageId BIGINT,
    ChannelId BIGINT,
    UserId BIGINT,
    Name VARCHAR(255),
    Content TEXT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_520_ci,
    Timestamp TIMESTAMP,
    PremiumType VARCHAR(50)
);

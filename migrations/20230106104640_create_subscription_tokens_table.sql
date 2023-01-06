-- Add migration script here
CREATE TABLE subscription_tokens(
    subsciption_token TEXT NOT NULL,
    subscriber_id uuid NOT NULL REFERENCES subscriptions (id),
    PRIMARY KEY (subsciption_token)
);
ALTER TABLE jira_tickets DROP FOREIGN KEY jira_tickets_ibfk_2;
ALTER TABLE jira_tickets ADD CONSTRAINT jira_tickets_ibfk_2 FOREIGN KEY (server_id) REFERENCES jira_servers(id) ON DELETE CASCADE;

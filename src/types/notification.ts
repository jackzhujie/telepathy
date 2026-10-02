export type NotificationType = 
  | 'model_download_complete'
  | 'app_update_available'
  | 'document_index_complete'
  | 'system';

export interface AppNotification {
  id: string;
  type: NotificationType;
  title: string;
  content: string;
  routePath?: string;
  isRead: boolean;
  createdAt: string;
}

export interface NotificationPayload {
  id: string;
  type: NotificationType;
  title: string;
  content: string;
  routePath?: string;
}

export interface Snap {
  id: string;
  content: string;
  tags: string | null;
  isPinned: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface PaginatedSnaps {
  items: Snap[];
  total: number;
  page: number;
  pageSize: number;
}

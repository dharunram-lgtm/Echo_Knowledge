export type RetryState =
  | 'pending'
  | 'processing'
  | 'retry_wait'
  | 'succeeded'
  | 'awaiting_key'
  | 'paused'
  | 'permanently_failed'
  | 'cancelled';

export interface RetryOverview {
  is_globally_paused: boolean;
  pending_indexing_count: number;
  retry_wait_indexing_count: number;
  succeeded_indexing_count: number;
  failed_indexing_count: number;
  pending_tagging_count: number;
  retry_wait_tagging_count: number;
  succeeded_tagging_count: number;
  failed_tagging_count: number;
}


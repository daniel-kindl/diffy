// The place of the user across a refresh (L-04). A refresh loads the same
// comparison again. The scroll position stays when it is the same comparison in
// the same view, and starts at the top when it is another one.

export interface ViewMemory {
  key: string;
  scrollTop: number;
}

/** What makes two loads the same place: the two sources and the view mode. */
export function viewKey(labels: readonly (string | undefined)[], unified: boolean): string {
  return `${unified ? "unified" : "split"}|${labels.map((l) => l ?? "").join("|")}`;
}

/** The scroll to restore for a new load, or 0 when it is another place. */
export function scrollToRestore(memory: ViewMemory | undefined, key: string): number {
  return memory && memory.key === key && memory.scrollTop > 0 ? memory.scrollTop : 0;
}

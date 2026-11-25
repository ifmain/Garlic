// Garlic GPF route ABI. A smaller score is preferred among healthy candidates.
int route_score(int latency_ms, int load_percent, int via_ingress) {
    int load_cost = load_percent * 3;
    return latency_ms > 2147483647 - load_cost ? 2147483647 : latency_ms + load_cost;
}

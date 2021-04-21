use engines::hbbft::test::hbbft_test_client::HbbftTestClient;
use std::collections::BTreeMap;

pub fn crank_network(clients: &mut Vec<HbbftTestClient>) {
    // sync blocks
    sync_blocks(clients);

    // sync transactions
    sync_transactions(clients);

    // sync consensus messages
    sync_consensus_messages(clients);
}

fn sync_blocks(clients: &mut Vec<HbbftTestClient>) {
    // Find client with most blocks.
    let best_client = clients
        .iter()
        .enumerate()
        .fold((0, 0u64), |prev, (index, client)| {
            // Get best block.
            let block_height = client.client.chain().best_block_number();
            // Check if best block is higher than current highest block.
            if block_height > prev.1 {
                (index, block_height)
            } else {
                prev
            }
        });

    let best = clients.iter().nth(best_client.0).unwrap().clone();

    for c in clients.iter_mut().enumerate() {
        if c.0 != best_client.0 {
            best.sync_blocks_to(c.1);
        }
    }
}

fn sync_transactions(clients: &mut Vec<HbbftTestClient>) {
    for c in clients.iter() {
        let cur = c.clone();
        for c2 in clients.iter() {
            let mut target = c2.clone();
            cur.sync_transactions_to(&mut target);
        }
    }
}

fn sync_consensus_messages(clients: &mut Vec<HbbftTestClient>) {
    let clients_map = clients
        .iter()
        .map(|c| (c.keypair.public().clone(), c))
        .collect::<BTreeMap<_, _>>();

    for (from, n) in &clients_map {
        let mut targeted_messages = n.notify.targeted_messages.write();
        for m in targeted_messages.drain(..) {
            clients_map
                .get(&m.1.expect("The Message target node id must be set"))
                .expect("Message target not found in nodes map")
                .client
                .engine()
                .handle_message(&m.0, Some(*from))
                .expect("Message handling to succeed");
        }
    }
}

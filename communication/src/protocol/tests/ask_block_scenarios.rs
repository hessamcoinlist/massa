use super::{mock_network_controller::MockNetworkController, tools};
use crate::network::NetworkCommand;
use crate::protocol::start_protocol_controller;
use crate::protocol::ProtocolEvent;
use crypto::signature::SignatureEngine;
use std::collections::HashSet;
use tools::assert_hash_asked_to_node;

#[tokio::test]
async fn test_without_a_priori() {
    // start
    let (protocol_config, serialization_context) = tools::create_protocol_config();

    let mut signature_engine = SignatureEngine::new();

    let (mut network_controller, network_command_sender, network_event_receiver) =
        MockNetworkController::new();

    // start protocol controller
    let (mut protocol_command_sender, protocol_event_receiver, protocol_manager) =
        start_protocol_controller(
            protocol_config.clone(),
            serialization_context.clone(),
            network_command_sender,
            network_event_receiver,
        )
        .await
        .expect("could not start protocol controller");

    let node_a = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();
    let node_b = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();
    let _node_c = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();

    // 2. Create a block coming from node 0.
    let block = tools::create_block(
        &node_a.private_key,
        &node_a.id.0,
        &serialization_context,
        &mut signature_engine,
    );
    let hash_1 = block
        .header
        .content
        .compute_hash(&serialization_context)
        .unwrap();
    // end set up

    // send wishlist
    protocol_command_sender
        .send_wishlist_delta(vec![hash_1].into_iter().collect(), HashSet::new())
        .await
        .unwrap();

    // assert it was asked to node A, then B
    assert_hash_asked_to_node(hash_1, node_a.id, &mut network_controller).await;
    assert_hash_asked_to_node(hash_1, node_b.id, &mut network_controller).await;

    // node B replied with the block
    network_controller.send_block(node_b.id, block).await;

    // 7. Make sure protocol did not send additional ask for block commands.
    let ask_for_block_cmd_filter = |cmd| match cmd {
        cmd @ NetworkCommand::AskForBlocks { .. } => Some(cmd),
        _ => None,
    };

    let got_more_commands = network_controller
        .wait_command(100.into(), ask_for_block_cmd_filter)
        .await;
    assert!(
        got_more_commands.is_none(),
        "unexpected command {:?}",
        got_more_commands
    );

    // Close everything
    protocol_manager
        .stop(protocol_event_receiver)
        .await
        .expect("Failed to shutdown protocol.");
}

#[tokio::test]
async fn test_someone_knows_it() {
    // start
    let (protocol_config, serialization_context) = tools::create_protocol_config();

    let mut signature_engine = SignatureEngine::new();

    let (mut network_controller, network_command_sender, network_event_receiver) =
        MockNetworkController::new();

    // start protocol controller
    let (mut protocol_command_sender, mut protocol_event_receiver, protocol_manager) =
        start_protocol_controller(
            protocol_config.clone(),
            serialization_context.clone(),
            network_command_sender,
            network_event_receiver,
        )
        .await
        .expect("could not start protocol controller");

    let node_a = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();
    let _node_b = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();
    let node_c = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();

    // 2. Create a block coming from node 0.
    let block = tools::create_block(
        &node_a.private_key,
        &node_a.id.0,
        &serialization_context,
        &mut signature_engine,
    );
    let hash_1 = block
        .header
        .content
        .compute_hash(&serialization_context)
        .unwrap();
    // end set up

    // node c must know about block
    network_controller
        .send_header(node_c.id, block.header.clone())
        .await;

    match protocol_event_receiver.wait_event().await.unwrap() {
        ProtocolEvent::ReceivedBlockHeader { hash: _, header: _ } => {}
        _ => panic!("unexpected protocol event"),
    };

    // send wishlist
    protocol_command_sender
        .send_wishlist_delta(vec![hash_1].into_iter().collect(), HashSet::new())
        .await
        .unwrap();

    assert_hash_asked_to_node(hash_1, node_c.id, &mut network_controller).await;

    // node C replied with the block
    network_controller.send_block(node_c.id, block).await;

    // 7. Make sure protocol did not send additional ask for block commands.
    let ask_for_block_cmd_filter = |cmd| match cmd {
        cmd @ NetworkCommand::AskForBlocks { .. } => Some(cmd),
        _ => None,
    };

    let got_more_commands = network_controller
        .wait_command(100.into(), ask_for_block_cmd_filter)
        .await;
    assert!(
        got_more_commands.is_none(),
        "unexpected command {:?}",
        got_more_commands
    );

    // Close everything
    protocol_manager
        .stop(protocol_event_receiver)
        .await
        .expect("Failed to shutdown protocol.");
}

#[tokio::test]
async fn test_dont_want_it_anymore() {
    // start
    let (protocol_config, serialization_context) = tools::create_protocol_config();

    let mut signature_engine = SignatureEngine::new();

    let (mut network_controller, network_command_sender, network_event_receiver) =
        MockNetworkController::new();

    // start protocol controller
    let (mut protocol_command_sender, protocol_event_receiver, protocol_manager) =
        start_protocol_controller(
            protocol_config.clone(),
            serialization_context.clone(),
            network_command_sender,
            network_event_receiver,
        )
        .await
        .expect("could not start protocol controller");

    let node_a = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();
    let _node_b = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();
    let _node_c = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();

    // 2. Create a block coming from node 0.
    let block = tools::create_block(
        &node_a.private_key,
        &node_a.id.0,
        &serialization_context,
        &mut signature_engine,
    );
    let hash_1 = block
        .header
        .content
        .compute_hash(&serialization_context)
        .unwrap();
    // end set up

    // send wishlist
    protocol_command_sender
        .send_wishlist_delta(vec![hash_1].into_iter().collect(), HashSet::new())
        .await
        .unwrap();

    // assert it was asked to node A
    assert_hash_asked_to_node(hash_1, node_a.id, &mut network_controller).await;

    // we don't want it anymore
    protocol_command_sender
        .send_wishlist_delta(HashSet::new(), vec![hash_1].into_iter().collect())
        .await
        .unwrap();

    // 7. Make sure protocol did not send additional ask for block commands.
    let ask_for_block_cmd_filter = |cmd| match cmd {
        cmd @ NetworkCommand::AskForBlocks { .. } => Some(cmd),
        _ => None,
    };

    let got_more_commands = network_controller
        .wait_command(100.into(), ask_for_block_cmd_filter)
        .await;
    assert!(
        got_more_commands.is_none(),
        "unexpected command {:?}",
        got_more_commands
    );

    // Close everything
    protocol_manager
        .stop(protocol_event_receiver)
        .await
        .expect("Failed to shutdown protocol.");
}

#[tokio::test]
async fn test_no_one_has_it() {
    // start
    let (protocol_config, serialization_context) = tools::create_protocol_config();

    let mut signature_engine = SignatureEngine::new();

    let (mut network_controller, network_command_sender, network_event_receiver) =
        MockNetworkController::new();

    // start protocol controller
    let (mut protocol_command_sender, protocol_event_receiver, protocol_manager) =
        start_protocol_controller(
            protocol_config.clone(),
            serialization_context.clone(),
            network_command_sender,
            network_event_receiver,
        )
        .await
        .expect("could not start protocol controller");

    let node_a = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();
    let node_b = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();
    let node_c = tools::create_and_connect_nodes(1, &signature_engine, &mut network_controller)
        .await
        .pop()
        .unwrap();

    // 2. Create a block coming from node 0.
    let block = tools::create_block(
        &node_a.private_key,
        &node_a.id.0,
        &serialization_context,
        &mut signature_engine,
    );
    let hash_1 = block
        .header
        .content
        .compute_hash(&serialization_context)
        .unwrap();
    // end set up

    // send wishlist
    protocol_command_sender
        .send_wishlist_delta(vec![hash_1].into_iter().collect(), HashSet::new())
        .await
        .unwrap();

    // assert it was asked to node A
    assert_hash_asked_to_node(hash_1, node_a.id, &mut network_controller).await;

    // node a replied is does not have it
    network_controller
        .send_block_not_found(node_a.id, hash_1)
        .await;

    assert_hash_asked_to_node(hash_1, node_b.id, &mut network_controller).await;
    assert_hash_asked_to_node(hash_1, node_c.id, &mut network_controller).await;
    assert_hash_asked_to_node(hash_1, node_a.id, &mut network_controller).await;
    assert_hash_asked_to_node(hash_1, node_b.id, &mut network_controller).await;
    assert_hash_asked_to_node(hash_1, node_c.id, &mut network_controller).await;

    // 7. Make sure protocol did not send additional ask for block commands.
    let ask_for_block_cmd_filter = |cmd| match cmd {
        cmd @ NetworkCommand::AskForBlocks { .. } => Some(cmd),
        _ => None,
    };

    let got_more_commands = network_controller
        .wait_command(100.into(), ask_for_block_cmd_filter)
        .await;
    assert!(
        got_more_commands.is_none(),
        "unexpected command {:?}",
        got_more_commands
    );

    // Close everything
    protocol_manager
        .stop(protocol_event_receiver)
        .await
        .expect("Failed to shutdown protocol.");
}
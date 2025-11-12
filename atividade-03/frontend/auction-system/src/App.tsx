import './App.css';
import React, { useState, useEffect, useRef } from 'react';
import { Bell, Gavel } from 'lucide-react';
import axios from 'axios';

// Types
interface Auction {
  id: number;
  item: string;
  start_timestamp: number;
  end_timestamp: string;
  status: boolean;
}

interface Notification {
  event_type: string;
  auction_id: number;
  data: string;
}

interface NotificationItem {
  id: number,
  event_type: string;
  auction_id: number;
  data: string;
}

const API_BASE_URL = 'http://localhost:8080/api';

const USER_ID = Math.floor(Math.random() * 5000);

const AuctionSystem: React.FC = () => {
  const [userId, setUserId] = useState<number>(-1);
  const [activeScreen, setActiveScreen] = useState<'auctions' | 'create'>('auctions');
  const [auctions, setAuctions] = useState<Auction[]>([]);
  const [notifications, setNotifications] = useState<NotificationItem[]>([]);
  const [subscribedAuctions, setSubscribedAuctions] = useState<Set<number>>(new Set());

  const [bidAuctionId, setBidAuctionId] = useState<string>('');
  const [bidAmount, setBidAmount] = useState<string>('');

  const [newAuction, setNewAuction] = useState({
    produto: '',
    descricao: '',
    inicioLeilao: '',
    fimLeilao: ''
  });

  let eventSource = null;

  useEffect(() => {
    fetchUserId();
  }, []);

  useEffect(() => {
    if (!userId) return;
    setupSSEConnection();
  }, [userId]);

  useEffect(() => {
    fetchAuctions();
    const interval = setInterval(fetchAuctions, 5000);
    return () => clearInterval(interval);
  }, []);

  const fetchUserId = async () => {
    setUserId(USER_ID);
  };

  const setupSSEConnection = () => {
    console.log('SSE connection would be established here for user:', userId);
    eventSource = new EventSource(`${API_BASE_URL}/api/v1/events?clientID=${USER_ID}`)

    eventSource.onmessage = (event) =>{
      let newNotification: Notification = JSON.parse(event.data);
      handleNotification(newNotification);
    };

    eventSource.onerror = (error) =>{
      console.log(`ERROR: ${error}`);
    };
    
  };

  const handleNotification = (newNotification: Notification) => {
    let something: NotificationItem = {
      id: notifications.length,
      ...newNotification,
    };

    setNotifications(prev => [something, ...prev]);
  };

  const fetchAuctions = async () => {
      let auctions: Auction[] = await axios.get(`${API_BASE_URL}/api/v1/auctions`);
      setAuctions(auctions);
  };

  const placeBid = async () => {
    if (!bidAuctionId || !bidAmount) return;
    console.log('Placing bid:', { auctionId: bidAuctionId, amount: bidAmount, userId });
    let res = axios.post(`${API_BASE_URL}/api/v1/bid`, {
      auction_id: bidAuctionId,
      client_id: USER_ID,
      value: Number(bidAmount),
      signature: "",
      public_key: "",
      valid: true,
    });
    console.log(res);

    setBidAuctionId('');
    setBidAmount('');
    fetchAuctions();
  };

  const subscribeToAuction = async (auctionId: number) => {
    console.log('Subscribing to auction:', auctionId);
    await axios.post(`${API_BASE_URL}/api/v1/subscribe`, {
      client_id: USER_ID,
      auctions: [auctionId]
    });
    setSubscribedAuctions(prev => new Set(prev).add(auctionId));
  };

  const unsubscribeFromAuction = async (auctionId: number) => {
    console.log('Unsubscribing from auction:', auctionId);
    await axios.post(`${API_BASE_URL}/api/v1/unsubscribe`, {
      client_id: USER_ID,
      auctions: [auctionId]
    });

    setSubscribedAuctions(prev => {
      const newSet = new Set(prev);
      newSet.delete(auctionId);
      return newSet;
    });
  };

  const createAuction = async () => {
    console.log('Creating auction:', newAuction);

    axios.post(`${API_BASE_URL}/api/v1/auction`, {
      item: newAuction.produto,
      start_timestamp: new Date(newAuction.inicioLeilao).getMilliseconds() || Date.now(),
      end_timestap: new Date(newAuction.fimLeilao).getMilliseconds() || Date.now() + 5 * 60 * 1000,
    });

    clearForm();
    fetchAuctions();
  };

  const clearForm = () => {
    setNewAuction({
      produto: '',
      descricao: '',
      inicioLeilao: '',
      fimLeilao: ''
    });
  };

  return (
    <div className="app-container">
      {/* Sidebar - Notifications */}
      <aside className="sidebar">
        <div className="sidebar-header">
          <Bell className="icon" />
          <h2>Notificações</h2>
        </div>
        <ul className="notification-list">
          {notifications.length === 0 ? (
            <li className="notification-empty">Nenhuma notificação</li>
          ) : (
            notifications.map(notification => (
              <li key={notification.id} className="notification-item">
                <p>{notification.data}</p>
              </li>
            ))
          )}
        </ul>
      </aside>

      {/* Main content */}
      <div className="main-content">
        {/* Top bar */}
        <header className="topbar">
          <div className="topbar-title">
            <Gavel className="icon" />
            <h1>Sistema de Leilões</h1>
          </div>
          <nav className="nav-buttons">
            <button
              className={`nav-btn ${activeScreen === 'auctions' ? 'active' : ''}`}
              onClick={() => setActiveScreen('auctions')}
            >
              Leilões Ativos
            </button>
            <button
              className={`nav-btn ${activeScreen === 'create' ? 'active' : ''}`}
              onClick={() => setActiveScreen('create')}
            >
              Criar Leilão
            </button>
          </nav>
          <div className="user-id">ID: {userId}</div>
        </header>

        {/* Screen content */}
        <main className="screen-content">
          {activeScreen === 'auctions' ? (
            <>
              {/* Bid form */}
              <section className="auction-form">
                <h2>Realizar Lance</h2>
                <div className="form-row">
                  <div className="form-group">
                    <label>ID do Leilão</label>
                    <input
                      type="number"
                      value={bidAuctionId}
                      onChange={e => setBidAuctionId(e.target.value)}
                      placeholder="Digite o ID"
                    />
                  </div>
                  <div className="form-group">
                    <label>Valor do Lance</label>
                    <input
                      type="number"
                      value={bidAmount}
                      onChange={e => setBidAmount(e.target.value)}
                      placeholder="Digite o valor"
                    />
                  </div>
                  <button onClick={placeBid} disabled={!bidAuctionId || !bidAmount}>
                    Enviar Lance
                  </button>
                </div>
              </section>

              {/* Auction list */}
              <section className="auction-list">
                <h2>Leilões Ativos</h2>
                {auctions.length === 0 ? (
                  <div className="empty-state">
                    <p>Nenhum leilão ativo no momento</p>
                    <button onClick={fetchAuctions}>Atualizar</button>
                  </div>
                ) : (
                  auctions.map(auction => (
                    <div key={auction.id} className="auction-item">
                      <div className="auction-info">
                        <h3>{auction.item}</h3>
                      </div>
                      <button
                        className={`subscribe-btn ${
                          subscribedAuctions.has(auction.id) ? 'active' : ''
                        }`}
                        onClick={() =>
                          subscribedAuctions.has(auction.id)
                            ? unsubscribeFromAuction(auction.id)
                            : subscribeToAuction(auction.id)
                        }
                      >
                        {subscribedAuctions.has(auction.id)
                          ? 'Inscrito'
                          : 'Inscrever'}
                      </button>
                    </div>
                  ))
                )}
              </section>
            </>
          ) : (
            <section className="auction-create">
              <h2>Criar Novo Leilão</h2>
              <form>
                <div className="form-group">
                  <label>Produto Leiloado</label>
                  <input
                    type="text"
                    value={newAuction.produto}
                    onChange={e => setNewAuction({ ...newAuction, produto: e.target.value })}
                  />
                </div>

                <div className="form-group">
                  <label>Descrição</label>
                  <textarea
                    value={newAuction.descricao}
                    onChange={e => setNewAuction({ ...newAuction, descricao: e.target.value })}
                  />
                </div>

                <div className="form-row">
                  <div className="form-group">
                    <label>Início do Leilão</label>
                    <input
                      type="datetime-local"
                      value={newAuction.inicioLeilao}
                      onChange={e => setNewAuction({ ...newAuction, inicioLeilao: e.target.value })}
                    />
                  </div>
                  <div className="form-group">
                    <label>Fim do Leilão</label>
                    <input
                      type="datetime-local"
                      value={newAuction.fimLeilao}
                      onChange={e => setNewAuction({ ...newAuction, fimLeilao: e.target.value })}
                    />
                  </div>
                </div>

                <div className="form-actions">
                  <button type="button" onClick={createAuction}>
                    Criar Leilão
                  </button>
                  <button type="button" onClick={clearForm}>
                    Limpar Campos
                  </button>
                </div>
              </form>
            </section>
          )}
        </main>
      </div>
    </div>
  );
};

export default AuctionSystem;

import './App.css';
import React, { useState, useEffect, useRef } from 'react';
import { Bell, Plus, Gavel } from 'lucide-react';

// Types
interface Auction {
  id: number;
  produto: string;
  descricao: string;
  inicioLeilao: string;
  fimLeilao: string;
  maiorLance: number;
}

interface Notification {
  id: string;
  message: string;
  timestamp: Date;
  type: 'bid' | 'ended' | 'payment' | 'info';
}

const API_BASE_URL = 'http://localhost:8080/api';

const AuctionSystem: React.FC = () => {
  const [userId, setUserId] = useState<string>('');
  const [activeScreen, setActiveScreen] = useState<'auctions' | 'create'>('auctions');
  const [auctions, setAuctions] = useState<Auction[]>([]);
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [subscribedAuctions, setSubscribedAuctions] = useState<Set<number>>(new Set());

  const [bidAuctionId, setBidAuctionId] = useState<string>('');
  const [bidAmount, setBidAmount] = useState<string>('');

  const [newAuction, setNewAuction] = useState({
    produto: '',
    descricao: '',
    inicioLeilao: '',
    fimLeilao: ''
  });

  const eventSourceRef = useRef<EventSource | null>(null);

  useEffect(() => {
    fetchUserId();
  }, []);

  useEffect(() => {
    if (!userId) return;
    setupSSEConnection();
    return () => eventSourceRef.current?.close();
  }, [userId]);

  useEffect(() => {
    fetchAuctions();
    const interval = setInterval(fetchAuctions, 5000);
    return () => clearInterval(interval);
  }, []);

  const fetchUserId = async () => {
    const mockUserId = `user-${Math.random().toString(36).substr(2, 9)}`;
    setUserId(mockUserId);
  };

  const setupSSEConnection = () => {
    console.log('SSE connection would be established here for user:', userId);
  };

  const handleNotification = (notification: any) => {
    const newNotification: Notification = {
      id: Math.random().toString(36),
      message: notification.message,
      timestamp: new Date(),
      type: notification.type || 'info'
    };
    setNotifications(prev => [newNotification, ...prev].slice(0, 50));
  };

  const fetchAuctions = async () => {
    console.log('Fetching active auctions...');
  };

  const placeBid = async () => {
    if (!bidAuctionId || !bidAmount) return;
    console.log('Placing bid:', { auctionId: bidAuctionId, amount: bidAmount, userId });
    setBidAuctionId('');
    setBidAmount('');
    fetchAuctions();
  };

  const subscribeToAuction = (auctionId: number) => {
    console.log('Subscribing to auction:', auctionId);
    setSubscribedAuctions(prev => new Set(prev).add(auctionId));
  };

  const unsubscribeFromAuction = (auctionId: number) => {
    console.log('Unsubscribing from auction:', auctionId);
    setSubscribedAuctions(prev => {
      const newSet = new Set(prev);
      newSet.delete(auctionId);
      return newSet;
    });
  };

  const createAuction = async () => {
    console.log('Creating auction:', newAuction);
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

  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat('pt-BR', {
      style: 'currency',
      currency: 'BRL'
    }).format(value);
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
                <p>{notification.message}</p>
                <span>{notification.timestamp.toLocaleTimeString('pt-BR')}</span>
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
                        <h3>{auction.produto}</h3>
                        <p>{auction.descricao}</p>
                        <p>Maior Lance: {formatCurrency(auction.maiorLance)}</p>
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

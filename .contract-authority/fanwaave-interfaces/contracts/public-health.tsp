namespace Fanwaave.Public;

/** Public, unauthenticated service health response shared by HTTP and RPC clients. */
model Health {
  /** True when the API process is serving requests. */
  ok: boolean;

  /** Stable service identity. */
  service: string;

  /** Fanwaave public protocol version. */
  protocol: string;
}

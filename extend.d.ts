//FROM - extend.d.ts

type Last<T> = T extends readonly [...any, infer R] ? R : never;

type RemoveLeadingChar<
  TLeading extends string,
  TString extends string
> = TString extends `${TLeading}${infer R}` ? R : TString;

export type ParamsFromRoute<T extends string> =
  T extends `${string}{${infer L}}${infer R}`
    ? {
        [key in RemoveLeadingChar<"*", L>]: string;
      } & ParamsFromRoute<R>
    : {};

export type AouHandler<TParams> = (
  req: AouRequest & { params: TParams }
) => Promise<AouResponse>;

export declare interface AouServer {
  get<TRoute extends string, TParams extends ParamsFromRoute<TRoute>>(
    route: TRoute,
    handler: (req: AouRequest & { params: TParams }) => Promise<AouResponse>
  ): void;
  head<TRoute extends string, TParams extends ParamsFromRoute<TRoute>>(
    route: TRoute,
    handler: (req: AouRequest & { params: TParams }) => Promise<AouResponse>
  ): void;
  post<TRoute extends string, TParams extends ParamsFromRoute<TRoute>>(
    route: TRoute,
    handler: (req: AouRequest & { params: TParams }) => Promise<AouResponse>
  ): void;
  put<TRoute extends string, TParams extends ParamsFromRoute<TRoute>>(
    route: TRoute,
    handler: (req: AouRequest & { params: TParams }) => Promise<AouResponse>
  ): void;
  delete<TRoute extends string, TParams extends ParamsFromRoute<TRoute>>(
    route: TRoute,
    handler: (req: AouRequest & { params: TParams }) => Promise<AouResponse>
  ): void;
  connect<TRoute extends string, TParams extends ParamsFromRoute<TRoute>>(
    route: TRoute,
    handler: (req: AouRequest & { params: TParams }) => Promise<AouResponse>
  ): void;
  options<TRoute extends string, TParams extends ParamsFromRoute<TRoute>>(
    route: TRoute,
    handler: (req: AouRequest & { params: TParams }) => Promise<AouResponse>
  ): void;
  trace<TRoute extends string, TParams extends ParamsFromRoute<TRoute>>(
    route: TRoute,
    handler: (req: AouRequest & { params: TParams }) => Promise<AouResponse>
  ): void;
  patch<TRoute extends string, TParams extends ParamsFromRoute<TRoute>>(
    route: TRoute,
    handler: (req: AouRequest & { params: TParams }) => Promise<AouResponse>
  ): void;
  all<TRoute extends string, TParams extends ParamsFromRoute<TRoute>>(
    route: TRoute,
    handler: (req: AouRequest & { params: TParams }) => Promise<AouResponse>
  ): void;
}

export declare class AouError {
  constructor(error: AouResponse);
}

export type MiddlewareHandler<T extends any, TNewContext extends any = any> = (
  req: AouRequest & {},
  context: T
) => Promise<{
  req: AouRequest;
  context: TNewContext;
}>;

export type AouHandlerWithContext<TParams, TContext> = (
  req: AouRequest & { params: TParams },
  context: TContext
) => Promise<AouResponse>;

export declare class AouMiddleware<
  const T extends MiddlewareHandler<any, any>[],
  TLast extends Last<T>,
  TLastContext extends TLast extends MiddlewareHandler<any, infer TContext>
    ? TContext
    : never
> {
  private readonly handlers: T;

  private constructor(handlers: T) {
    this.handlers = handlers;
  }

  static create<CTX, R>(handler: MiddlewareHandler<CTX, R>) {
    return new AouMiddleware([handler] as const);
  }

  public with<T extends MiddlewareHandler<TLastContext, any>>(handler: T) {
    return new AouMiddleware([...this.handlers, handler]);
  }

  public handle<TParams extends any>(
    handler: AouHandlerWithContext<TParams, TLastContext>
  ) {
    return async (req: AouRequest & { params: TParams }) => {
      let r: any = {
        req,
        context: {},
      };

      for (const middleware of this.handlers) {
        r = await middleware(r.req, r.context);
      }

      return handler(r.req, r.context);
    };
  }
}

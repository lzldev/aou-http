//FROM - extend.d.ts

type RemoveLeadingChar<
  TLeading extends string,
  TString extends string
> = TString extends `${TLeading}${infer R}` ? R : TString;

type ParamsFromRoute<T extends string> =
  T extends `${string}{${infer L}}${infer R}`
    ? {
        [key in RemoveLeadingChar<"*", L>]: string;
      } & ParamsFromRoute<R>
    : {};

declare interface AouServer {
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

declare class AouError {
  constructor(error: AouResponse);
}

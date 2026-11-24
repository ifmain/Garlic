.class public final synthetic Lorg/garlic/client/GarlicProxyService$2;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/GarlicProxyService;"
    method = "lambda$onStartCommand$1"
    proto = "(J)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/GarlicProxyService;

.field public final synthetic f$1:J


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/GarlicProxyService;J)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/GarlicProxyService$2;->f$0:Lorg/garlic/client/GarlicProxyService;

    iput-wide p2, p0, Lorg/garlic/client/GarlicProxyService$2;->f$1:J

    return-void
.end method


# virtual methods
.method public final run()V
    .locals 3

    .line 0
    iget-object v0, p0, Lorg/garlic/client/GarlicProxyService$2;->f$0:Lorg/garlic/client/GarlicProxyService;

    iget-wide v1, p0, Lorg/garlic/client/GarlicProxyService$2;->f$1:J

    invoke-virtual {v0, v1, v2}, Lorg/garlic/client/GarlicProxyService;->lambda$onStartCommand$1$org-garlic-client-GarlicProxyService(J)V

    return-void
.end method

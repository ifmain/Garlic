.class public final Lorg/garlic/client/ConnectionState;
.super Ljava/lang/Object;
.source "ConnectionState.java"


# instance fields
.field private active:Z

.field private desired:Z

.field private generation:J


# direct methods
.method public constructor <init>()V
    .locals 0

    .line 4
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    return-void
.end method

.method public static retryAllowed(Ljava/lang/String;)Z
    .locals 2

    .line 10
    const/4 v0, 0x1

    if-nez p0, :cond_0

    return v0

    .line 11
    :cond_0
    sget-object v1, Ljava/util/Locale;->ROOT:Ljava/util/Locale;

    invoke-virtual {p0, v1}, Ljava/lang/String;->toLowerCase(Ljava/util/Locale;)Ljava/lang/String;

    move-result-object p0

    .line 12
    const-string v1, "device_in_use"

    invoke-virtual {p0, v1}, Ljava/lang/String;->contains(Ljava/lang/CharSequence;)Z

    move-result v1

    if-nez v1, :cond_1

    const-string v1, "device access revoked"

    invoke-virtual {p0, v1}, Ljava/lang/String;->contains(Ljava/lang/CharSequence;)Z

    move-result v1

    if-nez v1, :cond_1

    .line 13
    const-string v1, "http 401"

    invoke-virtual {p0, v1}, Ljava/lang/String;->contains(Ljava/lang/CharSequence;)Z

    move-result v1

    if-nez v1, :cond_1

    const-string v1, "http 402"

    invoke-virtual {p0, v1}, Ljava/lang/String;->contains(Ljava/lang/CharSequence;)Z

    move-result v1

    if-nez v1, :cond_1

    const-string v1, "http 403"

    invoke-virtual {p0, v1}, Ljava/lang/String;->contains(Ljava/lang/CharSequence;)Z

    move-result v1

    if-nez v1, :cond_1

    .line 14
    const-string v1, "http 409"

    invoke-virtual {p0, v1}, Ljava/lang/String;->contains(Ljava/lang/CharSequence;)Z

    move-result v1

    if-nez v1, :cond_1

    const-string v1, "vpn permission is required"

    invoke-virtual {p0, v1}, Ljava/lang/String;->contains(Ljava/lang/CharSequence;)Z

    move-result v1

    if-nez v1, :cond_1

    .line 15
    const-string v1, "\u0441\u0435\u0442\u0435\u0432\u043e\u0439 \u043c\u043e\u0434\u0443\u043b\u044c \u043d\u0435\u0434\u043e\u0441\u0442\u0443\u043f\u0435\u043d"

    invoke-virtual {p0, v1}, Ljava/lang/String;->contains(Ljava/lang/CharSequence;)Z

    move-result p0

    if-nez p0, :cond_1

    goto :goto_0

    :cond_1
    const/4 v0, 0x0

    .line 12
    :goto_0
    return v0
.end method


# virtual methods
.method public declared-synchronized begin()J
    .locals 4

    monitor-enter p0

    .line 19
    const/4 v0, 0x1

    :try_start_0
    iput-boolean v0, p0, Lorg/garlic/client/ConnectionState;->desired:Z

    .line 20
    iget-boolean v1, p0, Lorg/garlic/client/ConnectionState;->active:Z
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    if-eqz v1, :cond_0

    monitor-exit p0

    const-wide/16 v0, -0x1

    return-wide v0

    .line 21
    :cond_0
    :try_start_1
    iput-boolean v0, p0, Lorg/garlic/client/ConnectionState;->active:Z

    .line 22
    iget-wide v0, p0, Lorg/garlic/client/ConnectionState;->generation:J

    const-wide/16 v2, 0x1

    add-long/2addr v0, v2

    iput-wide v0, p0, Lorg/garlic/client/ConnectionState;->generation:J
    :try_end_1
    .catchall {:try_start_1 .. :try_end_1} :catchall_0

    monitor-exit p0

    return-wide v0

    .line 18
    :catchall_0
    move-exception v0

    :try_start_2
    monitor-exit p0
    :try_end_2
    .catchall {:try_start_2 .. :try_end_2} :catchall_0

    throw v0
.end method

.method public declared-synchronized current(J)Z
    .locals 2

    monitor-enter p0

    .line 26
    :try_start_0
    iget-boolean v0, p0, Lorg/garlic/client/ConnectionState;->desired:Z

    if-eqz v0, :cond_0

    iget-boolean v0, p0, Lorg/garlic/client/ConnectionState;->active:Z

    if-eqz v0, :cond_0

    iget-wide v0, p0, Lorg/garlic/client/ConnectionState;->generation:J
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    cmp-long p1, v0, p1

    if-nez p1, :cond_0

    const/4 p1, 0x1

    goto :goto_0

    :cond_0
    const/4 p1, 0x0

    :goto_0
    monitor-exit p0

    return p1

    .line 26
    :catchall_0
    move-exception p1

    :try_start_1
    monitor-exit p0
    :try_end_1
    .catchall {:try_start_1 .. :try_end_1} :catchall_0

    throw p1
.end method

.method public declared-synchronized invalidate()V
    .locals 4

    monitor-enter p0

    .line 30
    const/4 v0, 0x0

    :try_start_0
    iput-boolean v0, p0, Lorg/garlic/client/ConnectionState;->active:Z

    .line 31
    iget-wide v0, p0, Lorg/garlic/client/ConnectionState;->generation:J

    const-wide/16 v2, 0x1

    add-long/2addr v0, v2

    iput-wide v0, p0, Lorg/garlic/client/ConnectionState;->generation:J
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    .line 32
    monitor-exit p0

    return-void

    .line 29
    :catchall_0
    move-exception v0

    :try_start_1
    monitor-exit p0
    :try_end_1
    .catchall {:try_start_1 .. :try_end_1} :catchall_0

    throw v0
.end method

.method public declared-synchronized stop()V
    .locals 1

    monitor-enter p0

    .line 35
    const/4 v0, 0x0

    :try_start_0
    iput-boolean v0, p0, Lorg/garlic/client/ConnectionState;->desired:Z

    .line 36
    invoke-virtual {p0}, Lorg/garlic/client/ConnectionState;->invalidate()V
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    .line 37
    monitor-exit p0

    return-void

    .line 34
    :catchall_0
    move-exception v0

    :try_start_1
    monitor-exit p0
    :try_end_1
    .catchall {:try_start_1 .. :try_end_1} :catchall_0

    throw v0
.end method
